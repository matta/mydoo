use crate::types::{TaskID, TunnelState};
use std::collections::HashSet;

/// Recursively collects all descendant IDs of a given task.
///
/// Performs a depth-first traversal to find all tasks that are direct or indirect children
/// of the specified `task_id`.
///
/// # Arguments
///
/// * `state` - The current tunnel state containing the task map.
/// * `task_id` - The ID of the task to find descendants for.
///
/// # Returns
///
/// A `HashSet` containing the IDs of all descendants. Returns an empty set if the task
/// has no children or does not exist.
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
///
/// Traverses parent links starting from the given task's immediate parent,
/// returning a list of ancestor IDs ordered from immediate parent to root.
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
///
/// Looks up the task in its parent's list and returns `None` if it is the first child
/// or if the task or its parent is not found.
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
