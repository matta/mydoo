use crate::adapter;
use anyhow::Result;
use automerge::Automerge;
use proptest::prelude::*;
use tasklens_core::{
    Action, TaskUpdates,
    types::{PlaceID, RepeatConfig, ScheduleType, TaskID, TaskStatus, TunnelState},
};

#[allow(dead_code)]
pub(super) static SETUP_PREFIXES: &[&str] = &["s-"];
#[allow(dead_code)]
pub(super) static REPLICA_A_PREFIXS: &[&str] = &["s-", "a-"];
#[allow(dead_code)]
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
    // 1. Hydrate according to strategy
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
            match adapter::hydrate_tunnel_state(doc) {
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

prop_compose! {
    pub(super) fn any_task_updates()(
        title in any::<Option<String>>(),
        status in any::<Option<TaskStatus>>(),
        place_id in any::<Option<Option<PlaceID>>>(),
        due_date in any::<Option<Option<i64>>>(),
        schedule_type in any::<Option<ScheduleType>>(),
        lead_time in any::<Option<i64>>(),
        repeat_config in any::<Option<Option<RepeatConfig>>>(),
        is_sequential in any::<Option<bool>>(),
        credits in any::<Option<f64>>(),
        desired_credits in any::<Option<f64>>(),
        credit_increment in any::<Option<f64>>(),
        importance in any::<Option<f64>>(),
        is_acknowledged in any::<Option<bool>>(),
        last_done in any::<Option<Option<i64>>>(),
        credits_timestamp in any::<Option<i64>>(),
    ) -> TaskUpdates {
        TaskUpdates {
            title,
            status,
            place_id,
            due_date,
            schedule_type,
            lead_time,
            repeat_config,
            is_sequential,
            credits,
            desired_credits,
            credit_increment,
            importance,
            is_acknowledged,
            last_done,
            credits_timestamp,
        }
    }
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

// --- Stateful Fuzzing Strategy ---

/// Tracks the state of the fuzz test simulation, specifically the set of
/// "Active" TaskIDs that have been created and not yet permanently deleted.
///
/// This allows the fuzz generator to meaningfully target existing tasks
/// (e.g., for Updates or Moves) rather than purely guessing random IDs.
#[derive(Clone, Debug)]
pub(super) struct FuzzState {
    pub active_ids: Vec<TaskID>,
    pub next_id_counter: usize,
}

impl Default for FuzzState {
    fn default() -> Self {
        Self {
            active_ids: Vec::new(),
            next_id_counter: 1,
        }
    }
}

/// Represents an abstract intention of an action, separated from the
/// concrete TaskIDs.
///
/// Instead of containing specific `TaskID`s (which might be invalid or unknown),
/// this struct uses "Selectors" (indices) which are resolved against the
/// current `FuzzState` at runtime to pick a valid target.
#[derive(Clone, Debug)]
pub(super) enum AbstractAction {
    Create {
        title_seed: String,
        parent_selector: Option<usize>,
    },
    Update {
        target_selector: usize,
        updates: TaskUpdates,
    },
    Delete {
        target_selector: usize,
    },
    Complete {
        target_selector: usize,
        time: i64,
    },
    Move {
        target_selector: usize,
        parent_selector: Option<usize>,
    },
    Refresh {
        time: i64,
    },
    // "Random" actions specifically targeting unknown IDs or edge cases
    Chaos(Action),
}

/// Generates a sequence of abstract actions with a bias towards "valid" operations.
///
/// * `valid_ratio`: The probability (0.0 to 1.0) that a generated action will
///   intent to target a valid, existing task (if any exist).
///
/// The strategy produces `AbstractAction`s which must be interpreted by
/// `interpret_actions` to produce concrete `Action`s.
pub(super) fn any_abstract_action(valid_ratio: f64) -> impl Strategy<Value = AbstractAction> {
    prop::bool::weighted(valid_ratio).prop_flat_map(move |valid| {
        if valid {
            // Generate a "Valid-intent" action (will try to use existing IDs)
            prop_oneof![
                // Create
                (any::<String>(), any::<Option<usize>>()).prop_map(|(t, p)| {
                    AbstractAction::Create {
                        title_seed: t,
                        parent_selector: p,
                    }
                }),
                // Update
                (any::<usize>(), any_task_updates()).prop_map(|(t, u)| AbstractAction::Update {
                    target_selector: t,
                    updates: u
                }),
                // Delete
                any::<usize>().prop_map(|t| AbstractAction::Delete { target_selector: t }),
                // Complete
                (any::<usize>(), any::<i64>()).prop_map(|(t, time)| AbstractAction::Complete {
                    target_selector: t,
                    time
                }),
                // Move
                (any::<usize>(), any::<Option<usize>>()).prop_map(|(t, p)| AbstractAction::Move {
                    target_selector: t,
                    parent_selector: p
                }),
                // Refresh
                any::<i64>().prop_map(|time| AbstractAction::Refresh { time }),
            ]
            .boxed()
        } else {
            // Generate a "Chaos" action (uses arbitrary strings/IDs)
            any_action().prop_map(AbstractAction::Chaos).boxed()
        }
    })
}

/// Interprets a sequence of `AbstractAction`s into concrete `Action`s using
/// a stateful context.
///
/// This function iterates through the abstract actions, maintaining the `FuzzState`
/// to track which TaskIDs are currently valid. It resolves "selectors" (indices)
/// from the abstract actions into actual `TaskID`s.
///
/// * `state`: The initial or current state of the simulation.
/// * `abstract_actions`: The sequence of intents to execute.
/// * `id_prefix`: A prefix for newly created TaskIDs to ensure uniqueness across
///   different replicas (e.g., "a-", "b-").
pub(super) fn interpret_actions(
    mut state: FuzzState,
    abstract_actions: Vec<AbstractAction>,
    id_prefix: &'static str,
) -> (Vec<Action>, FuzzState) {
    let mut actions = Vec::with_capacity(abstract_actions.len());

    for aa in abstract_actions {
        match aa {
            AbstractAction::Create {
                title_seed,
                parent_selector,
            } => {
                let id_str = format!("{}task-gen-{}", id_prefix, state.next_id_counter);
                state.next_id_counter += 1;
                let id = TaskID::from(id_str);

                let parent_id = if state.active_ids.is_empty() {
                    None
                } else {
                    parent_selector.map(|s| {
                        let idx = s % state.active_ids.len();
                        state.active_ids[idx].clone()
                    })
                };

                state.active_ids.push(id.clone());
                actions.push(Action::CreateTask {
                    id,
                    parent_id,
                    title: title_seed,
                });
            }
            AbstractAction::Update {
                target_selector,
                updates,
            } => {
                if state.active_ids.is_empty() {
                    // Fallback to no-op or chaos if no tasks
                    continue;
                }
                let idx = target_selector % state.active_ids.len();
                let id = state.active_ids[idx].clone();
                actions.push(Action::UpdateTask { id, updates });
            }
            AbstractAction::Delete { target_selector } => {
                if state.active_ids.is_empty() {
                    continue;
                }
                let idx = target_selector % state.active_ids.len();
                let id = state.active_ids.swap_remove(idx); // Remove from valid set
                actions.push(Action::DeleteTask { id });
            }
            AbstractAction::Complete {
                target_selector,
                time,
            } => {
                if state.active_ids.is_empty() {
                    continue;
                }
                let idx = target_selector % state.active_ids.len();
                let id = state.active_ids[idx].clone();
                actions.push(Action::CompleteTask {
                    id,
                    current_time: time,
                });
            }
            AbstractAction::Move {
                target_selector,
                parent_selector,
            } => {
                if state.active_ids.is_empty() {
                    continue;
                }
                let t_idx = target_selector % state.active_ids.len();
                let target_id = state.active_ids[t_idx].clone();

                let new_parent_id = parent_selector.and_then(|s| {
                    if state.active_ids.is_empty() {
                        None
                    } else {
                        let p_idx = s % state.active_ids.len();
                        Some(state.active_ids[p_idx].clone())
                    }
                });

                // Self-move prevention
                if let Some(ref npid) = new_parent_id
                    && npid == &target_id
                {
                    continue;
                }

                actions.push(Action::MoveTask {
                    id: target_id,
                    new_parent_id,
                });
            }
            AbstractAction::Refresh { time } => {
                actions.push(Action::RefreshLifecycle { current_time: time });
            }
            AbstractAction::Chaos(mut action) => {
                // If the chaos action creates a task, we must ensure its ID honors the
                // prefix to avoid collision between concurrent branches (mimicking UUIDs).
                if let Action::CreateTask { id, .. } = &mut action {
                    let new_id = format!("{}{}", id_prefix, id.as_str());
                    *id = TaskID::from(new_id);
                }
                actions.push(action);
            }
        }
    }

    (actions, state)
}
