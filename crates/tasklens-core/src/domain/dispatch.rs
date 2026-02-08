use automerge::transaction::Transactable;
use autosurgeon::{Doc, MaybeMissing};
use std::collections::HashMap;
use thiserror::Error;

use crate::{
    Action, PlaceUpdates, TaskUpdates,
    domain::{doc_bridge, lifecycle, routine_tasks},
    types::{
        PersistedTask, TaskID, TaskStatus, TunnelState, hydrate_f64, hydrate_option_f64,
        hydrate_option_i64, hydrate_option_maybe_missing,
    },
};

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),

    #[error("Hydrate error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),

    #[error("Hydration failed: {0}")]
    Hydration(String),

    #[error("Path key '{0}' is not an object")]
    InvalidPath(String),

    #[error("Task not found: {0}")]
    TaskNotFound(TaskID),

    #[error("Parent task not found: {0}")]
    ParentNotFound(TaskID),

    #[error("Task already exists: {0}")]
    TaskExists(TaskID),

    #[error("Place not found: {0}")]
    PlaceNotFound(crate::types::PlaceID),

    #[error("Cannot delete the built-in Anywhere place")]
    CannotDeleteAnywhere,

    #[error("Cycle detected moving task {0} to {1}")]
    CycleDetected(TaskID, TaskID),

    #[error("Cannot move task {0} to itself: {1}")]
    MoveToSelf(TaskID, TaskID),

    #[error("Inconsistency: {0}")]
    Inconsistency(String),

    #[error("Operation failed: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DispatchError>;

fn am_get<'a, T: Transactable>(
    doc: &'a T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<Option<(automerge::Value<'a>, automerge::ObjId)>> {
    doc.get(obj, prop).map_err(DispatchError::from)
}

fn am_delete<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
) -> Result<()> {
    doc.delete(obj, prop).map_err(DispatchError::from)
}

fn am_put_object<T: Transactable>(
    doc: &mut T,
    obj: &automerge::ObjId,
    prop: impl Into<automerge::Prop>,
    value: automerge::ObjType,
) -> Result<automerge::ObjId> {
    doc.put_object(obj, prop, value)
        .map_err(DispatchError::from)
}

/// Helper to ensure a path of map objects exists in the document.
///
/// Returns the `ObjId` of the final object in the path.
/// Creates intermediate maps if they are missing.
pub fn ensure_path<T: Transactable + Doc>(
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
            _ => return Err(DispatchError::InvalidPath(key.to_string())),
        };
    }
    Ok(current)
}

/// Hydrates a TunnelState from the current document, healing any structural inconsistencies.
pub fn hydrate_tunnel_state(doc: &impl autosurgeon::ReadDoc) -> Result<TunnelState> {
    let mut state = doc_bridge::hydrate_tunnel_state(doc).map_err(DispatchError::from)?;
    state.heal_structural_inconsistencies();
    Ok(state)
}

/// Runs an action on any Transactable + Doc object (Transaction, AutoCommit, etc.).
pub fn run_action(doc: &mut (impl Transactable + Doc), action: Action) -> Result<()> {
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
        Action::SetBalanceDistribution { distribution } => {
            handle_set_balance_distribution(doc, distribution)
        }
        Action::CreatePlace {
            id,
            name,
            hours,
            included_places,
        } => handle_create_place(doc, id, name, hours, included_places),
        Action::UpdatePlace { id, updates } => handle_update_place(doc, id, updates),
        Action::DeletePlace { id } => handle_delete_place(doc, id),
    }
}

fn handle_set_balance_distribution(
    doc: &mut (impl Transactable + Doc),
    distribution: HashMap<TaskID, f64>,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    for (task_id, desired_credits) in distribution {
        if let Some((automerge::Value::Object(automerge::ObjType::Map), task_obj_id)) =
            am_get(doc, &tasks_obj_id, task_id.as_str())?
        {
            autosurgeon::reconcile_prop(doc, &task_obj_id, "desiredCredits", desired_credits)
                .map_err(DispatchError::from)?;
        }
    }
    Ok(())
}

fn handle_create_place(
    doc: &mut (impl Transactable + Doc),
    id: crate::types::PlaceID,
    name: String,
    hours: String,
    included_places: Vec<crate::types::PlaceID>,
) -> Result<()> {
    let places_obj_id = ensure_path(doc, &automerge::ROOT, vec!["places"])?;

    let place = crate::types::Place {
        id: id.clone(),
        name,
        hours,
        included_places,
    };

    autosurgeon::reconcile_prop(doc, &places_obj_id, id.as_str(), &place)
        .map_err(DispatchError::from)?;

    Ok(())
}

fn handle_update_place(
    doc: &mut (impl Transactable + Doc),
    id: crate::types::PlaceID,
    updates: PlaceUpdates,
) -> Result<()> {
    let places_obj_id = ensure_path(doc, &automerge::ROOT, vec!["places"])?;

    let place_obj_id = match am_get(doc, &places_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), obj_id)) => obj_id,
        _ => return Err(DispatchError::PlaceNotFound(id)),
    };

    if let Some(name) = updates.name {
        autosurgeon::reconcile_prop(doc, &place_obj_id, "name", name)
            .map_err(DispatchError::from)?;
    }
    if let Some(hours) = updates.hours {
        autosurgeon::reconcile_prop(doc, &place_obj_id, "hours", hours)
            .map_err(DispatchError::from)?;
    }
    if let Some(included_places) = updates.included_places {
        autosurgeon::reconcile_prop(doc, &place_obj_id, "includedPlaces", &included_places)
            .map_err(DispatchError::from)?;
    }

    Ok(())
}

fn handle_delete_place(
    doc: &mut (impl Transactable + Doc),
    id: crate::types::PlaceID,
) -> Result<()> {
    if id.as_str() == crate::types::ANYWHERE_PLACE_ID {
        return Err(DispatchError::CannotDeleteAnywhere);
    }

    let places_obj_id = ensure_path(doc, &automerge::ROOT, vec!["places"])?;

    if am_get(doc, &places_obj_id, id.as_str())?.is_none() {
        return Err(DispatchError::PlaceNotFound(id));
    }

    am_delete(doc, &places_obj_id, id.as_str())?;

    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;
    let state = hydrate_tunnel_state(doc)?;
    for (task_id, task) in &state.tasks {
        if task.place_id.as_ref() == Some(&id) {
            let task_obj_id = match am_get(doc, &tasks_obj_id, task_id.as_str())? {
                Some((automerge::Value::Object(automerge::ObjType::Map), obj_id)) => obj_id,
                _ => continue,
            };
            am_delete(doc, &task_obj_id, "placeId")?;
        }
    }

    for place in state.places.values() {
        if place.included_places.contains(&id) {
            let place_obj_id = match am_get(doc, &places_obj_id, place.id.as_str())? {
                Some((automerge::Value::Object(automerge::ObjType::Map), obj_id)) => obj_id,
                _ => continue,
            };
            let cleaned: Vec<crate::types::PlaceID> = place
                .included_places
                .iter()
                .filter(|p| *p != &id)
                .cloned()
                .collect();
            autosurgeon::reconcile_prop(doc, &place_obj_id, "includedPlaces", &cleaned)
                .map_err(DispatchError::from)?;
        }
    }

    Ok(())
}

fn handle_create_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    parent_id: Option<TaskID>,
    title: String,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let parent = if let Some(pid) = &parent_id {
        let p: MaybeMissing<PersistedTask> =
            autosurgeon::hydrate_prop(doc, &tasks_obj_id, pid.as_str())?;
        match p {
            MaybeMissing::Present(task) => Some(task),
            MaybeMissing::Missing => {
                return Err(DispatchError::ParentNotFound(pid.clone()));
            }
        }
    } else {
        None
    };

    let exists = automerge::ReadDoc::get(doc, &tasks_obj_id, id.as_str())?.is_some();
    if exists {
        return Err(DispatchError::TaskExists(id.clone()));
    }

    let task = crate::create_new_task(id.clone(), title, parent.as_ref());

    autosurgeon::reconcile_prop(doc, &tasks_obj_id, id.as_str(), &task)
        .map_err(DispatchError::from)?;

    if let Some(pid) = parent_id {
        let parent_obj_id = ensure_path(doc, &tasks_obj_id, vec![pid.as_str()])?;

        let mut child_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, &parent_obj_id, "childTaskIds") {
                Ok(ids) => match ids {
                    MaybeMissing::Missing => Vec::new(),
                    MaybeMissing::Present(ids) => ids,
                },
                Err(e) => return Err(DispatchError::Hydrate(e)),
            };

        if !child_ids.contains(&id) {
            child_ids.push(id);
            autosurgeon::reconcile_prop(doc, &parent_obj_id, "childTaskIds", &child_ids)
                .map_err(DispatchError::from)?;
        }
    } else {
        let mut root_ids: Vec<TaskID> =
            match autosurgeon::hydrate_prop(doc, automerge::ROOT, "rootTaskIds") {
                Ok(ids) => match ids {
                    MaybeMissing::Missing => Vec::new(),
                    MaybeMissing::Present(ids) => ids,
                },
                Err(e) => return Err(DispatchError::Hydrate(e)),
            };

        if !root_ids.contains(&id) {
            root_ids.push(id);
            autosurgeon::reconcile_prop(doc, automerge::ROOT, "rootTaskIds", &root_ids)
                .map_err(DispatchError::from)?;
        }
    }

    Ok(())
}

fn handle_update_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    updates: TaskUpdates,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(DispatchError::TaskNotFound(id.clone())),
    };

    if let Some(title) = updates.title {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "title", title)
            .map_err(DispatchError::from)?;
    }
    if let Some(notes) = updates.notes {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "notes", notes)
            .map_err(DispatchError::from)?;
    }
    if let Some(status) = updates.status {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "status", status)
            .map_err(DispatchError::from)?;
    }
    if let Some(place_id_update) = updates.place_id {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "placeId", place_id_update)
            .map_err(DispatchError::from)?;
    }

    if let Some(is_seq) = updates.is_sequential {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "isSequential", is_seq)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.credits {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "credits", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.desired_credits {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "desiredCredits", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.credit_increment {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "creditIncrement", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.importance {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "importance", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.is_acknowledged {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "isAcknowledged", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(repeat_config_update) = updates.repeat_config {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "repeatConfig", repeat_config_update)
            .map_err(DispatchError::from)?;
    }
    if let Some(val) = updates.credits_timestamp {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "creditsTimestamp", val)
            .map_err(DispatchError::from)?;
    }
    if let Some(ts) = updates.priority_timestamp {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "priorityTimestamp", ts)
            .map_err(DispatchError::from)?;
    }

    if updates.due_date.is_some()
        || updates.schedule_type.is_some()
        || updates.lead_time.is_some()
        || updates.last_done.is_some()
    {
        let schedule_obj_id = ensure_path(doc, &task_obj_id, vec!["schedule"])?;

        if let Some(due_date_update) = updates.due_date {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "dueDate", due_date_update)
                .map_err(DispatchError::from)?;
        }
        if let Some(schedule_type) = updates.schedule_type {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "type", schedule_type)
                .map_err(DispatchError::from)?;
        }
        if let Some(lead_time_update) = updates.lead_time {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "leadTime", lead_time_update)
                .map_err(DispatchError::from)?;
        }
        if let Some(last_done_update) = updates.last_done {
            autosurgeon::reconcile_prop(doc, &schedule_obj_id, "lastDone", last_done_update)
                .map_err(DispatchError::from)?;
        }
    }

    Ok(())
}

fn handle_delete_task(doc: &mut (impl Transactable + Doc), id: TaskID) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(DispatchError::TaskNotFound(id.clone())),
    };

    let parent_id: Option<TaskID> =
        hydrate_option_maybe_missing(doc, &task_obj_id, autosurgeon::Prop::Key("parentId".into()))?;

    let child_ids: Vec<TaskID> = match autosurgeon::hydrate_prop(doc, &task_obj_id, "childTaskIds")?
    {
        MaybeMissing::Present(ids) => ids,
        MaybeMissing::Missing => Vec::new(),
    };

    for cid in child_ids {
        if let Err(e) = handle_delete_task(doc, cid) {
            match e {
                DispatchError::TaskNotFound(_) => {}
                _ => return Err(e),
            }
        }
    }

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

    am_delete(doc, &tasks_obj_id, id.as_str())?;

    Ok(())
}

/// Half-life for credit decay in milliseconds (7 days).
const CREDITS_HALF_LIFE_MS: f64 = 604_800_000.0;

fn handle_complete_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    current_time: i64,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    // 1. Get Target Task Object & Credit Increment
    let target_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(DispatchError::TaskNotFound(id.clone())),
    };

    let credit_increment: Option<f64> = hydrate_option_f64(
        doc,
        &target_obj_id,
        autosurgeon::Prop::Key("creditIncrement".into()),
    )?;
    let increment_val = credit_increment.unwrap_or(0.5);

    // 3. Update Credits (Target Only)
    let credits: f64 = hydrate_f64(
        doc,
        &target_obj_id,
        autosurgeon::Prop::Key("credits".into()),
    )?;
    let credits_timestamp: i64 = hydrate_option_i64(
        doc,
        &target_obj_id,
        autosurgeon::Prop::Key("creditsTimestamp".into()),
    )?
    .unwrap_or(0);

    let time_delta_ms = current_time.saturating_sub(credits_timestamp) as f64;
    let decay_factor = 0.5_f64.powf(time_delta_ms / CREDITS_HALF_LIFE_MS);
    let decayed_credits = credits * decay_factor;

    let new_credits = decayed_credits + increment_val;

    autosurgeon::reconcile_prop(doc, &target_obj_id, "credits", new_credits)
        .map_err(DispatchError::from)?;
    autosurgeon::reconcile_prop(doc, &target_obj_id, "creditsTimestamp", current_time)
        .map_err(DispatchError::from)?;

    // 4. Update Status (Target Only)
    autosurgeon::reconcile_prop(doc, &target_obj_id, "status", TaskStatus::Done)
        .map_err(DispatchError::from)?;
    autosurgeon::reconcile_prop(doc, &target_obj_id, "lastCompletedAt", Some(current_time))
        .map_err(DispatchError::from)?;

    Ok(())
}

fn handle_move_task(
    doc: &mut (impl Transactable + Doc),
    id: TaskID,
    new_parent_id: Option<TaskID>,
) -> Result<()> {
    let tasks_obj_id = ensure_path(doc, &automerge::ROOT, vec!["tasks"])?;

    let task_obj_id = match am_get(doc, &tasks_obj_id, id.as_str())? {
        Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
        _ => return Err(DispatchError::TaskNotFound(id.clone())),
    };

    let old_parent_id: Option<TaskID> =
        hydrate_option_maybe_missing(doc, &task_obj_id, autosurgeon::Prop::Key("parentId".into()))?;

    if old_parent_id == new_parent_id {
        return Ok(());
    }

    {
        let state = doc_bridge::hydrate_tunnel_state(doc).map_err(DispatchError::from)?;
        if let Some(ref npid) = new_parent_id {
            if !state.tasks.contains_key(npid) {
                return Err(DispatchError::ParentNotFound(npid.clone()));
            }
            if npid == &id {
                return Err(DispatchError::MoveToSelf(id.clone(), npid.clone()));
            }
            if causes_cycle(&state.tasks, &id, npid) {
                return Err(DispatchError::CycleDetected(id.clone(), npid.clone()));
            }
        }
    }

    if let Some(opid) = old_parent_id {
        let op_obj_id = match am_get(doc, &tasks_obj_id, opid.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => {
                return Err(DispatchError::Inconsistency(format!(
                    "Old parent disappeared: {}",
                    opid
                )));
            }
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

    if let Some(npid) = new_parent_id.clone() {
        let np_obj_id = match am_get(doc, &tasks_obj_id, npid.as_str())? {
            Some((automerge::Value::Object(automerge::ObjType::Map), id)) => id,
            _ => {
                return Err(DispatchError::Inconsistency(format!(
                    "New parent disappeared: {}",
                    npid
                )));
            }
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

    if let Some(npid) = new_parent_id {
        autosurgeon::reconcile_prop(doc, &task_obj_id, "parentId", &npid)?;
    } else {
        am_delete(doc, &task_obj_id, "parentId")?;
    }

    Ok(())
}

fn handle_refresh_lifecycle(doc: &mut (impl Transactable + Doc), current_time: i64) -> Result<()> {
    let mut state = hydrate_tunnel_state(doc)?;
    lifecycle::acknowledge_completed_tasks(&mut state);
    routine_tasks::wake_up_routine_tasks(&mut state, current_time);
    doc_bridge::reconcile_tunnel_state(doc, &state).map_err(DispatchError::from)?;
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
mod tests {
    use super::*;
    use crate::domain::doc_bridge;
    use crate::types::{ANYWHERE_PLACE_ID, PlaceID};
    use automerge::AutoCommit;

    /// Creates an initialized AutoCommit doc with empty TunnelState.
    fn new_doc() -> AutoCommit {
        let mut doc = AutoCommit::new();
        let initial = TunnelState {
            tasks: HashMap::new(),
            places: HashMap::new(),
            root_task_ids: Vec::new(),
            metadata: None,
        };
        doc_bridge::reconcile_tunnel_state(&mut doc, &initial).unwrap();
        doc
    }

    fn create_place(doc: &mut AutoCommit, id: &str) {
        run_action(
            doc,
            Action::CreatePlace {
                id: PlaceID::from(id),
                name: id.to_string(),
                hours: r#"{"mode":"always_open"}"#.to_string(),
                included_places: vec![],
            },
        )
        .unwrap();
    }

    #[test]
    fn update_place_name() {
        let mut doc = new_doc();
        create_place(&mut doc, "office");

        run_action(
            &mut doc,
            Action::UpdatePlace {
                id: PlaceID::from("office"),
                updates: PlaceUpdates {
                    name: Some("Main Office".to_string()),
                    ..Default::default()
                },
            },
        )
        .unwrap();

        let state = hydrate_tunnel_state(&doc).unwrap();
        let place = state.places.get(&PlaceID::from("office")).unwrap();
        assert_eq!(place.name, "Main Office");
    }

    #[test]
    fn update_place_hours_and_included() {
        let mut doc = new_doc();
        create_place(&mut doc, "office");
        create_place(&mut doc, "desk-a");

        run_action(
            &mut doc,
            Action::UpdatePlace {
                id: PlaceID::from("office"),
                updates: PlaceUpdates {
                    hours: Some(r#"{"mode":"always_closed"}"#.to_string()),
                    included_places: Some(vec![PlaceID::from("desk-a")]),
                    ..Default::default()
                },
            },
        )
        .unwrap();

        let state = hydrate_tunnel_state(&doc).unwrap();
        let place = state.places.get(&PlaceID::from("office")).unwrap();
        assert_eq!(place.hours, r#"{"mode":"always_closed"}"#);
        assert_eq!(place.included_places, vec![PlaceID::from("desk-a")]);
    }

    #[test]
    fn update_nonexistent_place_fails() {
        let mut doc = new_doc();

        let result = run_action(
            &mut doc,
            Action::UpdatePlace {
                id: PlaceID::from("ghost"),
                updates: PlaceUpdates {
                    name: Some("nope".to_string()),
                    ..Default::default()
                },
            },
        );

        assert!(matches!(result, Err(DispatchError::PlaceNotFound(_))));
    }

    #[test]
    fn delete_place_removes_it() {
        let mut doc = new_doc();
        create_place(&mut doc, "office");

        run_action(
            &mut doc,
            Action::DeletePlace {
                id: PlaceID::from("office"),
            },
        )
        .unwrap();

        let state = hydrate_tunnel_state(&doc).unwrap();
        assert!(!state.places.contains_key(&PlaceID::from("office")));
    }

    #[test]
    fn delete_place_clears_task_place_ids() {
        let mut doc = new_doc();
        create_place(&mut doc, "office");

        let task_id = TaskID::from("t1");
        run_action(
            &mut doc,
            Action::CreateTask {
                id: task_id.clone(),
                parent_id: None,
                title: "test task".to_string(),
            },
        )
        .unwrap();
        run_action(
            &mut doc,
            Action::UpdateTask {
                id: task_id.clone(),
                updates: TaskUpdates {
                    place_id: Some(Some(PlaceID::from("office"))),
                    ..Default::default()
                },
            },
        )
        .unwrap();

        run_action(
            &mut doc,
            Action::DeletePlace {
                id: PlaceID::from("office"),
            },
        )
        .unwrap();

        let state = hydrate_tunnel_state(&doc).unwrap();
        let task = state.tasks.get(&task_id).unwrap();
        assert_eq!(task.place_id, None);
    }

    #[test]
    fn delete_place_cleans_included_places() {
        let mut doc = new_doc();
        create_place(&mut doc, "desk-a");
        create_place(&mut doc, "building");

        run_action(
            &mut doc,
            Action::UpdatePlace {
                id: PlaceID::from("building"),
                updates: PlaceUpdates {
                    included_places: Some(vec![PlaceID::from("desk-a")]),
                    ..Default::default()
                },
            },
        )
        .unwrap();

        run_action(
            &mut doc,
            Action::DeletePlace {
                id: PlaceID::from("desk-a"),
            },
        )
        .unwrap();

        let state = hydrate_tunnel_state(&doc).unwrap();
        let building = state.places.get(&PlaceID::from("building")).unwrap();
        assert!(building.included_places.is_empty());
    }

    #[test]
    fn delete_nonexistent_place_fails() {
        let mut doc = new_doc();

        let result = run_action(
            &mut doc,
            Action::DeletePlace {
                id: PlaceID::from("ghost"),
            },
        );

        assert!(matches!(result, Err(DispatchError::PlaceNotFound(_))));
    }

    #[test]
    fn delete_anywhere_place_fails() {
        let mut doc = new_doc();

        let result = run_action(
            &mut doc,
            Action::DeletePlace {
                id: PlaceID::from(ANYWHERE_PLACE_ID),
            },
        );

        assert!(matches!(result, Err(DispatchError::CannotDeleteAnywhere)));
    }
}
