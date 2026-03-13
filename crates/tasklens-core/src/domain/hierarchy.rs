use crate::types::{TaskID, TunnelState};
use std::collections::HashSet;

/// Recursively collects all descendant IDs of a given task.
///
/// Performs a depth-first traversal of the `state` (the current tunnel state containing the task map)
/// to find all tasks that are direct or indirect children of the specified `task_id`.
/// Returns a [`HashSet`] containing the IDs of all descendants, or an empty set
/// if the task has no children or does not exist.
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::hierarchy::get_descendant_ids;
/// use tasklens_core::types::{TunnelState, TaskID};
/// use std::collections::HashMap;
///
/// let state = TunnelState {
///     tasks: HashMap::new(),
///     root_task_ids: vec![],
///     places: HashMap::new(),
///     metadata: None,
/// };
///
/// let task_id = TaskID::from("1");
/// let descendants = get_descendant_ids(&state, &task_id);
/// assert!(descendants.is_empty());
/// ```
pub fn get_descendant_ids(state: &TunnelState, task_id: &TaskID) -> HashSet<TaskID> {
    let mut descendants = HashSet::new();
    let mut stack = Vec::new();

    if let Some(task) = state.tasks.get(task_id) {
        stack.extend(task.child_task_ids.iter().cloned());
    }

    while let Some(current_id) = stack.pop() {
        if descendants.insert(current_id.clone())
            && let Some(task) = state.tasks.get(&current_id)
        {
            stack.extend(task.child_task_ids.iter().cloned());
        }
    }

    descendants
}

/// Collects all ancestor IDs up to the root.
pub fn get_ancestor_ids(state: &TunnelState, task_id: &TaskID) -> Vec<TaskID> {
    let mut ancestors = Vec::new();
    let mut current_id = task_id.clone();

    while let Some(task) = state.tasks.get(&current_id) {
        if let Some(parent_id) = &task.parent_id {
            ancestors.push(parent_id.clone());
            current_id = parent_id.clone();
        } else {
            break;
        }
    }

    ancestors
}

/// Returns the previous sibling in the parent's `child_task_ids` list.
pub fn get_previous_sibling(state: &TunnelState, task_id: &TaskID) -> Option<TaskID> {
    let task = state.tasks.get(task_id)?;

    let sibling_ids = if let Some(parent_id) = &task.parent_id {
        let parent = state.tasks.get(parent_id)?;
        &parent.child_task_ids
    } else {
        &state.root_task_ids
    };

    let pos = sibling_ids.iter().position(|id| id == task_id)?;
    if pos > 0 {
        Some(sibling_ids[pos - 1].clone())
    } else {
        None
    }
}
