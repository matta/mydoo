use crate::actions::{Action, TaskUpdates};
use crate::adapter;
use anyhow::Result;
use automerge::Automerge;
use automerge::ReadDoc;
use proptest::prelude::*;
use tasklens_core::types::{PlaceID, RepeatConfig, ScheduleType, TaskID, TaskStatus, TunnelState};

pub(super) fn init_doc() -> Result<Automerge> {
    let mut doc = Automerge::new();
    let id = crate::doc_id::DocumentId::new();
    adapter::init_state(&mut doc, &id)?;
    Ok(doc)
}

pub(super) fn check_invariants(doc: &Automerge) -> Result<(), String> {
    // 1. Manual map integrity check (detecting "phantom" objects created by ensure_path)
    if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), tasks_id))) =
        doc.get(&automerge::ROOT, "tasks")
    {
        for task_key in doc.keys(&tasks_id) {
            if let Ok(Some((automerge::Value::Object(automerge::ObjType::Map), task_obj))) =
                doc.get(&tasks_id, &task_key)
            {
                // Fields that should NEVER be missing if a task entry exists
                for field in ["id", "title", "status"] {
                    if doc.get(&task_obj, field).unwrap_or(None).is_none() {
                        return Err(format!(
                            "Broken Invariant path: tasks[\"{}\"] exists but is missing mandatory field \"{}\". (Likely a phantom object hazard)",
                            task_key, field
                        ));
                    }
                }
            }
        }
    }

    // 2. Check full hydration
    let state: TunnelState = match adapter::hydrate(doc) {
        Ok(s) => s,
        Err(e) => {
            let realized = crate::debug_utils::inspect_automerge_doc_full(doc);
            return Err(format!(
                "Hydration broken: {}\n\nFull Document Structure:\n{:#?}",
                e, realized
            ));
        }
    };

    // 3. Check Logical Invariants (Parent/Child consistency)
    for (id, task) in &state.tasks {
        if let Some(pid) = &task.parent_id {
            match state.tasks.get(pid) {
                Some(parent) => {
                    if !parent.child_task_ids.contains(id) {
                        return Err(format!(
                            "Inconsistency path: tasks[\"{}\"].parentId -> \"{}\" BUT tasks[\"{}\"].childTaskIds missing \"{}\"",
                            id, pid, pid, id
                        ));
                    }
                }
                None => {
                    return Err(format!(
                        "Broken Link path: tasks[\"{}\"].parentId -> \"{}\" BUT task \"{}\" does not exist in map",
                        id, pid, pid
                    ));
                }
            }
        }

        for cid in &task.child_task_ids {
            match state.tasks.get(cid) {
                Some(child) => {
                    if child.parent_id.as_ref() != Some(id) {
                        return Err(format!(
                            "Inconsistency path: tasks[\"{}\"].childTaskIds [contains \"{}\"] BUT tasks[\"{}\"].parentId is detected as {:?}",
                            id, cid, cid, child.parent_id
                        ));
                    }
                }
                None => {
                    return Err(format!(
                        "Broken Link path: tasks[\"{}\"].childTaskIds [contains \"{}\"] BUT child \"{}\" does not exist in map",
                        id, cid, cid
                    ));
                }
            }
        }
    }

    Ok(())
}

pub(super) fn any_task_id() -> impl Strategy<Value = TaskID> {
    prop_oneof![
        Just(TaskID::from("task-1")),
        Just(TaskID::from("task-2")),
        Just(TaskID::from("task-3")),
        Just(TaskID::from("task-4")),
        Just(TaskID::from("task-5")),
    ]
}

pub(super) fn any_place_id() -> impl Strategy<Value = PlaceID> {
    prop_oneof![
        Just(PlaceID::from("place-1")),
        Just(PlaceID::from("place-2")),
    ]
}

pub(super) fn any_task_updates() -> impl Strategy<Value = TaskUpdates> {
    (
        any::<Option<String>>(),
        any::<Option<TaskStatus>>(),
        any::<Option<Option<PlaceID>>>(),
        any::<Option<Option<i64>>>(),
        any::<Option<ScheduleType>>(),
        any::<Option<i64>>(),
        any::<Option<Option<RepeatConfig>>>(),
        any::<Option<bool>>(),
    )
        .prop_map(
            |(
                title,
                status,
                place_id,
                due_date,
                schedule_type,
                lead_time,
                repeat_config,
                is_sequential,
            )| {
                TaskUpdates {
                    title,
                    status,
                    place_id,
                    due_date,
                    schedule_type,
                    lead_time,
                    repeat_config,
                    is_sequential,
                }
            },
        )
}

pub(super) fn any_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (any_task_id(), any::<Option<TaskID>>(), any::<String>()).prop_map(
            |(id, parent_id, title)| {
                Action::CreateTask {
                    id,
                    parent_id,
                    title,
                }
            }
        ),
        (any_place_id(), any::<String>())
            .prop_map(|(_id, _name)| { Action::RefreshLifecycle { current_time: 0 } }),
        (any_task_id(), any_task_updates())
            .prop_map(|(id, updates)| { Action::UpdateTask { id, updates } }),
        any_task_id().prop_map(|id| Action::DeleteTask { id }),
        (any_task_id(), any::<i64>())
            .prop_map(|(id, current_time)| { Action::CompleteTask { id, current_time } }),
        (any_task_id(), any::<Option<TaskID>>())
            .prop_map(|(id, new_parent_id)| { Action::MoveTask { id, new_parent_id } }),
        any::<i64>().prop_map(|current_time| Action::RefreshLifecycle { current_time }),
    ]
}
