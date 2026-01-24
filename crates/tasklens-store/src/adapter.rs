use crate::actions::{Action, TaskUpdates};
use anyhow::{Result, anyhow};
use automerge::ReadDoc;
use automerge::transaction::Transactable;
use autosurgeon::{Doc, MaybeMissing, reconcile};
use std::collections::HashMap;

use crate::doc_id::TaskLensUrl;
use tasklens_core::types::{
    DocMetadata, PersistedTask, TaskID, TaskStatus, TunnelState, hydrate_optional_task_id,
};

fn am_get<'a, T: Transactable>(
    doc: &'a T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<Option<(automerge::Value<'a>, automerge::ObjId)>, automerge::AutomergeError> {
    doc.get(obj, prop)
}

fn am_delete<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<(), automerge::AutomergeError> {
    doc.delete(obj, prop)
}

fn am_put_object<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
    value: automerge::ObjType,
) -> Result<automerge::ObjId, automerge::AutomergeError> {
    doc.put_object(obj, prop, value)
}

/// Helper to ensure a path of map objects exists in the document.
///
/// Returns the `ObjId` of the final object in the path.
/// Creates intermediate maps if they are missing.
pub(crate) fn ensure_path<T: Transactable + Doc>(
    doc: &mut T,
    root: &automerge::ObjId,
    path: Vec<&str>,
) -> Result<automerge::ObjId> {
    let mut current = root.clone();
    for key in path {
        let val = am_get(doc, &current, key)?;
        current = match val {
            Some((automerge::Value::Object(_), id)) => id,
            None => am_put_object(doc, &current, key, automerge::ObjType::Map)?,
            _ => return Err(anyhow!("Path key '{}' is not an object", key)),
        };
    }
    Ok(current)
}

/// Hydrates a Rust struct from the current document or handle.
pub(crate) fn hydrate<T: autosurgeon::Hydrate>(doc: &impl autosurgeon::ReadDoc) -> Result<T> {
    autosurgeon::hydrate(doc).map_err(|e| anyhow!("Hydration failed: {}", e))
}

pub(crate) fn init_state(
    doc: &mut automerge::Automerge,
    id: &crate::doc_id::DocumentId,
) -> Result<()> {
    let mut tx = doc.transaction();

    let initial_state = TunnelState {
        next_task_id: 1,
        next_place_id: 1,
        tasks: HashMap::new(),
        places: HashMap::new(),
        root_task_ids: Vec::new(),
        metadata: Some(DocMetadata {
            automerge_url: Some(TaskLensUrl::from(id.clone()).to_string()),
        }),
    };

    if let Err(e) = reconcile(&mut tx, &initial_state) {
        tracing::error!("Failed to reconcile initial state: {}", e);
        return Err(anyhow!("Failed to reconcile initial state: {}", e));
    }
    tx.commit();
    Ok(())
}

/// Reconciles a Rust struct with the current document.
pub(crate) fn expensive_reconcile<T: autosurgeon::Reconcile>(
    doc: &mut automerge::Automerge,
    data: &T,
) -> Result<(), autosurgeon::ReconcileError> {
    let mut tx = doc.transaction();
    reconcile(&mut tx, data)?;
    tx.commit();
    Ok(())
}

/// A "total hack" repair utility that fixes tasks with "DoDonee" status,
/// changing them to "Done".
pub(crate) fn repair_dodonee(doc: &mut automerge::Automerge) -> Result<()> {
    let mut tx = doc.transaction();

    // 1. Get tasks map
    let tasks_obj_id = match am_get(&tx, &automerge::ROOT, "tasks")? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Ok(()),
    };

    // 2. Iterate keys and find those needing repair
    let mut tasks_to_repair = Vec::new();
    {
        let keys = tx.keys(&tasks_obj_id);
        for task_id in keys {
            let task_obj_id = match am_get(&tx, &tasks_obj_id, task_id.as_str())? {
                Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
                _ => continue,
            };

            // Check status
            let status_val = am_get(&tx, &task_obj_id, "status")?;
            let is_dodonee = match status_val {
                Some((automerge::Value::Scalar(scalar), _)) => match scalar.as_ref() {
                    automerge::ScalarValue::Str(s) => s.as_str() == "DoDonee",
                    _ => false,
                },
                Some((automerge::Value::Object(automerge::ObjType::Text), id)) => {
                    tx.text(id)? == "DoDonee"
                }
                _ => false,
            };

            if is_dodonee {
                tasks_to_repair.push(task_obj_id);
            }
        }
    }

    // 3. Repair them
    for task_obj_id in tasks_to_repair {
        tracing::info!("Repairing task {:?}: DoDonee -> Done", task_obj_id);
        // Set it to Done.
        autosurgeon::reconcile_prop(&mut tx, task_obj_id, "status", TaskStatus::Done)
            .map_err(|e| anyhow!("Repair reconciliation failed: {}", e))?;
    }
    tx.commit();
    Ok(())
}

/// Dispatches an action to modify the application state.
pub(crate) fn dispatch(doc: &mut automerge::Automerge, action: Action) -> Result<()> {
    let mut tx = doc.transaction();
    let res = run_action(&mut tx, action);
    tx.commit();
    res
}

/// Lower-level action handler that works on any transactable object.
pub(crate) fn run_action(doc: &mut (impl Transactable + Doc), action: Action) -> Result<()> {
    match action {
        Action::CreateTask {
            id,
            parent_id,
            title,
        } => handle_create_task(doc, id, parent_id, title),
        Action::UpdateTask { id, updates } => handle_update_task(doc, id, updates),
        Action::DeleteTask { id } => handle_delete_task(doc, id),
        Action::CompleteTask { id, current_time } => handle_complete_task(doc, id, current_time),
        Action::MoveTask { id, new_parent_id } => handle_move_task(doc, id, new_parent_id),
        Action::RefreshLifecycle { current_time } => handle_refresh_lifecycle(doc, current_time),
    }
}

pub(crate) fn handle_create_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    parent_id: Option<TaskID>,
    title: String,
) -> Result<()> {
    // 1. Get Tasks Map.
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    // 2. Resolve parent task and validate existence.
    let parent = if let Some(pid) = &parent_id {
        let p: MaybeMissing<PersistedTask> =
            autosurgeon::hydrate_prop(doc, &tasks_obj_id, pid.as_str())
                .map_err(|e| anyhow!("Failed to hydrate parent task: {}", e))?;
        match p {
            MaybeMissing::Present(task) => Some(task),
            MaybeMissing::Missing => {
                return Err(anyhow!(
                    "Cannot create task with non-existent parent: {}",
                    pid
                ));
            }
        }
    } else {
        None
    };

    // 2b. Check if task already exists.
    let exists = automerge::ReadDoc::get(doc, &tasks_obj_id, id.as_str())?.is_some();
    if exists {
        return Err(anyhow!("Task already exists: {}", id));
    }

    // 3. Create the new task struct.
    let task = tasklens_core::create_new_task(id.clone(), title, parent.as_ref());

    // 4. Reconcile the new task map entry.
    autosurgeon::reconcile_prop(doc, &tasks_obj_id, id.as_str(), &task)
        .map_err(|e| anyhow!("Failed to reconcile new task: {}", e))?;

    if let Some(pid) = parent_id {
        // Get parent object ID.
        let parent_obj_id = ensure_path(doc, &tasks_obj_id, vec![pid.as_str()])?;

        // Hydrate current children list.
        let mut child_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds") {
                Ok(ids) => match ids {
                    MaybeMissing::Missing => Vec::new(),
                    MaybeMissing::Present(ids) => ids,
                },
                Err(e) => return Err(anyhow!("Failed to hydrate child ids: {}", e)),
            };

        if !child_ids.contains(&id) {
            child_ids.push(id);
            // Reconcile updated children list.
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(|e| anyhow!("Failed to reconcile child ids: {}", e))?;
        }
    } else {
        // Update root task list.
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, automerge::ROOT, "rootTaskIds") {
                Ok(ids) => match ids {
                    MaybeMissing::Missing => Vec::new(),
                    MaybeMissing::Present(ids) => ids,
                },
                Err(e) => return Err(anyhow!("Failed to hydrate root task ids: {}", e)),
            };

        if !root_ids.contains(&id) {
            root_ids.push(id);
            autosurgeon::reconcile_prop(doc, automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(|e| anyhow!("Failed to reconcile root task ids: {}", e))?;
        }
    }

    Ok(())
}

pub(crate) fn handle_update_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    updates: TaskUpdates,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(anyhow!("Task not found: {}", id)),
    };

    if let Some(title) = updates.title {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "title", title)
            .map_err(|e| anyhow!("Failed to update title: {}", e))?;
    }
    if let Some(status) = updates.status {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "status", status)
            .map_err(|e| anyhow!("Failed to update status: {}", e))?;
    }
    if let Some(place_id_update) = updates.place_id {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "placeId", place_id_update)
            .map_err(|e| anyhow!("Failed to update placeId: {}", e))?;
    }

    if updates.due_date.is_some() || updates.schedule_type.is_some() || updates.lead_time.is_some()
    {
        let schedule_obj_id = ensure_path(doc, &task_obj_id, vec!["schedule"])?;

        if let Some(due_date_update) = updates.due_date {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "dueDate", due_date_update)
                .map_err(|e| anyhow!("Failed to update dueDate: {}", e))?;
        }
        if let Some(schedule_type) = updates.schedule_type {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "type", schedule_type)
                .map_err(|e| anyhow!("Failed to update schedule type: {}", e))?;
        }
        if let Some(lead_time_update) = updates.lead_time {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "leadTime", lead_time_update)
                .map_err(|e| anyhow!("Failed to update leadTime: {}", e))?;
        }
    }

    if let Some(repeat_config_update) = updates.repeat_config {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "repeatConfig", repeat_config_update)
            .map_err(|e| anyhow!("Failed to update repeatConfig: {}", e))?;
    }
    if let Some(is_seq) = updates.is_sequential {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "isSequential", is_seq)
            .map_err(|e| anyhow!("Failed to update isSequential: {}", e))?;
    }

    Ok(())
}

pub(crate) fn handle_delete_task(doc: &mut (impl Transactable + Doc), id: TaskID) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    // 1. Get task object ID.
    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(anyhow!("Task not found: {}", id)),
    };

    // 2. Resolve parent and children.
    let parent_id: Option<TaskID> =
        hydrate_optional_task_id(doc, &task_obj_id, autosurgeon::Prop::Key("parentId".into()))?;

    let child_ids: Vec<TaskID> = match autosurgeon::hydrate_prop(doc, &task_obj_id, "childTaskIds")?
    {
        MaybeMissing::Present(ids) => ids,
        MaybeMissing::Missing => Vec::new(),
    };

    // 3. Promote children to roots.
    if !child_ids.is_empty() {
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };

        for cid in child_ids {
            if let Some((automerge::Value::Object(automerge::ObjType::Map), child_task_obj_id)) =
                am_get(doc, &tasks_obj_id, cid.as_str())?
            {
                am_delete(doc, &child_task_obj_id, "parentId")?;
                if !root_ids.contains(&cid) {
                    root_ids.push(cid);
                }
            }
        }
        autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &root_ids)?;
    }

    // 4. Remove from parent or root list.
    if let Some(pid) = parent_id {
        if let Some((automerge::Value::Object(automerge::ObjType::Map), parent_obj_id)) =
            am_get(doc, &tasks_obj_id, pid.as_str())?
        {
            let mut p_child_ids: Vec<TaskID> =
                match autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds")? {
                    MaybeMissing::Present(ids) => ids,
                    MaybeMissing::Missing => Vec::new(),
                };
            p_child_ids.retain(|cid| cid != &id);
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &p_child_ids)?;
        }
    } else {
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };
        root_ids.retain(|rid| rid != &id);
        autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &root_ids)?;
    }

    // 5. Delete the task itself.
    am_delete(doc, &tasks_obj_id, id.as_str())?;

    Ok(())
}

pub(crate) fn handle_complete_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    current_time: i64,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(anyhow!("Task not found: {}", id)),
    };

    autosurgeon::reconcile_prop(doc, &task_obj_id, "status", TaskStatus::Done)
        .map_err(|e| anyhow!("Failed to update status: {}", e))?;

    autosurgeon::reconcile_prop(doc, &task_obj_id, "lastCompletedAt", Some(current_time))
        .map_err(|e| anyhow!("Failed to update lastCompletedAt: {}", e))?;

    Ok(())
}

pub(crate) fn handle_move_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    new_parent_id: Option<TaskID>,
) -> Result<()> {
    // 1. Resolve tasks map
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    // 2. Resolve task object
    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(anyhow!("Cannot move non-existent task: {}", id)),
    };

    // 3. Resolve old parent ID
    let old_parent_id: Option<TaskID> =
        hydrate_optional_task_id(doc, &task_obj_id, autosurgeon::Prop::Key("parentId".into()))?;

    // 4. Shortcut if same
    if old_parent_id == new_parent_id {
        return Ok(());
    }

    // 5. Validation: cycle detection still needs a hydrated partial state (for now).
    // We only hydrate what we need for the cycle check.
    {
        // Still using a full hydrate here for simplicity of cycle detection,
        // but it doesn't affect the reconciliation below.
        let state: TunnelState = autosurgeon::hydrate(doc)?;
        if let Some(ref npid) = new_parent_id {
            if !state.tasks.contains_key(npid) {
                return Err(anyhow!("Cannot move to non-existent parent: {}", npid));
            }
            if npid == &id {
                return Err(anyhow!("Cannot move task to itself: {}", id));
            }
            if causes_cycle(&state.tasks, &id, npid) {
                return Err(anyhow!("Cannot move task: cycle detected"));
            }
        }
    }

    // 6. Perform the updates in Automerge.

    // 6a. Remove from old container (parent's child list or root list).
    if let Some(opid) = old_parent_id {
        let op_obj_id = match am_get(doc, &tasks_obj_id, opid.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => return Err(anyhow!("Old parent disappeared: {}", opid)),
        };

        let mut child_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &op_obj_id, "childTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };
        child_ids.retain(|cid| cid != &id);
        autosurgeon::reconcile_prop(doc, &op_obj_id, "childTaskIds", &child_ids)?;
    } else {
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };
        root_ids.retain(|rid| rid != &id);
        autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &root_ids)?;
    }

    // 6b. Add to new container (parent's child list or root list).
    if let Some(npid) = new_parent_id.clone() {
        let np_obj_id = match am_get(doc, &tasks_obj_id, npid.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => return Err(anyhow!("New parent disappeared: {}", npid)),
        };

        let mut child_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &np_obj_id, "childTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };
        if !child_ids.contains(&id) {
            child_ids.push(id.clone());
            autosurgeon::reconcile_prop(doc, &np_obj_id, "childTaskIds", &child_ids)?;
        }
    } else {
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &automerge::ROOT, "rootTaskIds")? {
                MaybeMissing::Present(ids) => ids,
                MaybeMissing::Missing => Vec::new(),
            };
        if !root_ids.contains(&id) {
            root_ids.push(id.clone());
            autosurgeon::reconcile_prop(doc, &automerge::ROOT, "rootTaskIds", &root_ids)?;
        }
    }

    // 6c. Update task's parentId field.
    if let Some(npid) = new_parent_id {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "parentId", &npid)?;
    } else {
        am_delete(doc, &task_obj_id, "parentId")?;
    }

    Ok(())
}

pub(crate) fn handle_refresh_lifecycle(
    doc: &mut (impl Transactable + Doc),
    current_time: i64,
) -> Result<()> {
    let mut state: TunnelState = hydrate(doc)?;
    tasklens_core::domain::lifecycle::acknowledge_completed_tasks(&mut state);
    tasklens_core::domain::routine_tasks::wake_up_routine_tasks(&mut state, current_time);
    reconcile(doc, &state).map_err(|e| anyhow!("Dispatch reconciliation failed: {}", e))?;
    Ok(())
}

fn causes_cycle(
    tasks: &HashMap<TaskID, PersistedTask>,
    task_id: &TaskID,
    new_parent_id: &TaskID,
) -> bool {
    let mut current = Some(new_parent_id);
    while let Some(curr) = current {
        if curr == task_id {
            return true;
        }
        current = tasks.get(curr).and_then(|t| t.parent_id.as_ref());
    }
    false
}

#[cfg(test)]
mod tests;
