use crate::actions::{Action, TaskUpdates};
use crate::adapter;
use anyhow::Result;
use automerge::Automerge;
use automerge::ReadDoc;
use proptest::prelude::*;
use tasklens_core::types::{PlaceID, RepeatConfig, ScheduleType, TaskID, TaskStatus, TunnelState};

pub(super) static SETUP_PREFIXES: &[&str] = &["s-"];
pub(super) static REPLICA_A_PREFIXS: &[&str] = &["s-", "a-"];
pub(super) static REPLICA_B_PREFIXS: &[&str] = &["s-", "b-"];

pub(super) enum HydrationStrategy {
    /// Use autosurgeon::hydrate directly, failing if there are any structural inconsistencies.
    Strict,
    /// Use adapter::hydrate which automatically heals structural inconsistencies.
    Heal,
}

pub(super) fn init_doc() -> Result<Automerge> {
    let mut doc = Automerge::new();
    let id = crate::doc_id::DocumentId::new();
    adapter::init_state(&mut doc, &id)?;
    Ok(doc)
}

pub(super) fn dispatch_and_validate(doc: &mut Automerge, action: Action, context: &str) {
    match adapter::dispatch(doc, action) {
        Ok(_) => {}
        Err(
            adapter::AdapterError::TaskNotFound(_)
            | adapter::AdapterError::ParentNotFound(_)
            | adapter::AdapterError::TaskExists(_)
            | adapter::AdapterError::CycleDetected(..)
            | adapter::AdapterError::MoveToSelf(..),
        ) => {
            // Domain errors are expected in fuzz tests since we generate random actions.
        }
        Err(e) => panic!("Unexpected error in {}: {:?}", context, e),
    }
}

pub(super) fn check_invariants(doc: &Automerge, strategy: HydrationStrategy) -> Result<(), String> {
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

    // 2. Check hydration according to strategy
    let state: TunnelState = match strategy {
        HydrationStrategy::Strict => {
            // We use autosurgeon::hydrate directly here because we WANT to see inconsistencies.
            match autosurgeon::hydrate(doc) {
                Ok(s) => s,
                Err(e) => {
                    let realized = crate::debug_utils::inspect_automerge_doc_full(doc);
                    return Err(format!(
                        "Hydration broken (Strict): {}\n\nFull Document Structure:\n{:#?}",
                        e, realized
                    ));
                }
            }
        }
        HydrationStrategy::Heal => {
            // Use the adapter's hydrate which heals structural issues.
            match adapter::hydrate(doc) {
                Ok(s) => s,
                Err(e) => {
                    let realized = crate::debug_utils::inspect_automerge_doc_full(doc);
                    return Err(format!(
                        "Hydration broken (Heal): {}\n\nFull Document Structure:\n{:#?}",
                        e, realized
                    ));
                }
            }
        }
    };

    // 3. Structural Integrity & Cycle Detection
    let mut seen_in_lists = std::collections::HashSet::with_capacity(state.tasks.len());
    let mut root_set = std::collections::HashSet::with_capacity(state.root_task_ids.len());

    // Step A: Validate root_task_ids uniqueness and existence
    for rid in &state.root_task_ids {
        if !state.tasks.contains_key(rid) {
            return Err(format!(
                "Broken Link: root_task_ids contains \"{}\" but task does not exist",
                rid
            ));
        }
        if !seen_in_lists.insert(rid) {
            return Err(format!(
                "Invalid Tree: Task \"{}\" appears more than once in roots",
                rid
            ));
        }
        root_set.insert(rid);

        // Root tasks must not have a parent_id
        if let Some(pid) = &state.tasks[rid].parent_id {
            return Err(format!(
                "Inconsistency: Root task \"{}\" has parent \"{}\"",
                rid, pid
            ));
        }
    }

    // Step B: Validate child list uniqueness and parent consistency
    for (id, task) in &state.tasks {
        let mut child_set = std::collections::HashSet::with_capacity(task.child_task_ids.len());
        for cid in &task.child_task_ids {
            let child_task = match state.tasks.get(cid) {
                Some(t) => t,
                None => {
                    return Err(format!(
                        "Broken Link: Task \"{}\" child \"{}\" does not exist",
                        id, cid
                    ));
                }
            };

            if !child_set.insert(cid) {
                return Err(format!(
                    "Invalid Tree: Task \"{}\" appears more than once in child list of \"{}\"",
                    cid, id
                ));
            }
            if !seen_in_lists.insert(cid) {
                return Err(format!(
                    "Invalid Tree: Task \"{}\" has multiple parents or is both a root and a child",
                    cid
                ));
            }

            // Child MUST have this task as its parent_id
            if child_task.parent_id.as_ref() != Some(id) {
                return Err(format!(
                    "Inconsistency: Task \"{}\" claims \"{}\" as child, but \"{}\" parent_id is {:?}",
                    id, cid, cid, child_task.parent_id
                ));
            }
        }

        // Point 1 & 3 Verification: Inverse Parent Link Consistency
        if let Some(pid) = &task.parent_id {
            // If it has a parent_id, it MUST exist in the parent's child list
            let parent_task = match state.tasks.get(pid) {
                Some(pt) => pt,
                None => {
                    return Err(format!(
                        "Broken Link: Task \"{}\" has parent \"{}\" but parent does not exist (Broken Parent Point 2 equivalent)",
                        id, pid
                    ));
                }
            };

            if !parent_task.child_task_ids.contains(id) {
                return Err(format!(
                    "Inconsistency: Task \"{}\" has parent \"{}\", but \"{}\" does not have \"{}\" as child",
                    id, pid, pid, id
                ));
            }
        } else {
            // If a task has no parent_id, it MUST be in root_task_ids (Point 1)
            if !root_set.contains(id) {
                return Err(format!(
                    "Inconsistency: Task \"{}\" has no parent but is missing from root_task_ids",
                    id
                ));
            }
        }
    }

    // Step C: Check for orphaned tasks (in map but not in any list)
    if seen_in_lists.len() != state.tasks.len() {
        let first_orphaned = state
            .tasks
            .keys()
            .find(|id| !seen_in_lists.contains(id))
            .unwrap();
        return Err(format!(
            "Structural Leak: {} tasks in map but only {} reachable via root/child hierarchy. First orphaned: {}",
            state.tasks.len(),
            seen_in_lists.len(),
            first_orphaned
        ));
    }

    // Step D: Detect "floating" cycles (disconnected components)
    // Every node has exactly 1 parent (if not root) and we verified all nodes are in exactly one list.
    // The only remaining failure mode is a closed loop that is not reachable from the roots.
    let mut reachable = std::collections::HashSet::with_capacity(state.tasks.len());
    let mut stack = state.root_task_ids.clone();
    while let Some(id) = stack.pop() {
        if let Some(t) = state.tasks.get(&id) {
            for cid in &t.child_task_ids {
                stack.push(cid.clone());
            }
        }
        let id_str = id.to_string();
        if !reachable.insert(id) {
            // This should be technically impossible due to the uniqueness checks in Step A & B,
            // but we check it for completeness.
            return Err(format!("Cycle detected at \"{}\"", id_str));
        }
    }

    if reachable.len() != state.tasks.len() {
        let first_unreachable = state
            .tasks
            .keys()
            .find(|id| !reachable.contains(*id))
            .unwrap();
        return Err(format!(
            "Cycle detected! Task \"{}\" and potentially others are part of a closed loop unreachable from roots",
            first_unreachable
        ));
    }

    Ok(())
}

#[expect(dead_code)]
pub(super) fn any_task_id() -> impl Strategy<Value = TaskID> {
    any_task_id_for_prefix("")
}

pub(super) fn any_task_id_for_prefix(p: &'static str) -> impl Strategy<Value = TaskID> {
    prop_oneof![
        Just(TaskID::from(format!("{}task-1", p))),
        Just(TaskID::from(format!("{}task-2", p))),
        Just(TaskID::from(format!("{}task-3", p))),
        Just(TaskID::from(format!("{}task-4", p))),
        Just(TaskID::from(format!("{}task-5", p))),
    ]
}

pub(super) fn any_task_id_for_prefixes(
    prefixes: &'static [&'static str],
) -> impl Strategy<Value = TaskID> {
    let mut strategies = Vec::new();
    for p in prefixes {
        strategies.push(any_task_id_for_prefix(p).boxed());
    }
    proptest::strategy::Union::new(strategies)
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

pub(super) fn any_optional_task_id_for_prefixes(
    prefixes: &'static [&'static str],
) -> impl Strategy<Value = Option<TaskID>> {
    prop_oneof![
        Just(None),
        any_task_id_for_prefixes(prefixes).prop_map(Some),
    ]
}

pub(super) fn any_action() -> impl Strategy<Value = Action> {
    any_action_for_replica("", &[""])
}

pub(super) fn any_action_for_replica(
    create_prefix: &'static str,
    target_prefixes: &'static [&'static str],
) -> impl Strategy<Value = Action> {
    prop_oneof![
        (
            any_task_id_for_prefix(create_prefix),
            any_optional_task_id_for_prefixes(target_prefixes),
            any::<String>()
        )
            .prop_map(|(id, parent_id, title)| {
                Action::CreateTask {
                    id,
                    parent_id,
                    title,
                }
            }),
        (any_place_id(), any::<String>())
            .prop_map(|(_id, _name)| { Action::RefreshLifecycle { current_time: 0 } }),
        (
            any_task_id_for_prefixes(target_prefixes),
            any_task_updates()
        )
            .prop_map(|(id, updates)| { Action::UpdateTask { id, updates } }),
        any_task_id_for_prefixes(target_prefixes).prop_map(|id| Action::DeleteTask { id }),
        (any_task_id_for_prefixes(target_prefixes), any::<i64>())
            .prop_map(|(id, current_time)| { Action::CompleteTask { id, current_time } }),
        (
            any_task_id_for_prefixes(target_prefixes),
            any_optional_task_id_for_prefixes(target_prefixes)
        )
            .prop_map(|(id, new_parent_id)| { Action::MoveTask { id, new_parent_id } }),
        any::<i64>().prop_map(|current_time| Action::RefreshLifecycle { current_time }),
    ]
}
