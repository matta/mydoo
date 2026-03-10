use crate::types::{TaskID, TunnelState};
use std::collections::HashSet;

/// Recursively collects all descendant IDs of a given task.
///
/// Performs a depth-first traversal of `state` to find all tasks that are direct or indirect children
/// of the specified `task_id`. Returns a [`HashSet`] containing the IDs of all descendants, or an
/// empty set if the task has no children or does not exist.
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::hierarchy::get_descendant_ids;
/// use tasklens_core::types::{TaskID, TunnelState, PersistedTask, TaskStatus, Schedule, ScheduleType};
/// use std::collections::HashMap;
///
/// let parent_id = TaskID::from("parent");
/// let child_id = TaskID::from("child");
///
/// let mut parent_task = PersistedTask {
///     id: parent_id.clone(),
///     title: "Parent".to_string(),
///     notes: String::new(),
///     parent_id: None,
///     child_task_ids: vec![child_id.clone()],
///     place_id: None,
///     status: TaskStatus::Pending,
///     importance: 1.0,
///     credit_increment: None,
///     credits: 0.0,
///     desired_credits: 0.0,
///     credits_timestamp: 0,
///     priority_timestamp: 0,
///     schedule: Schedule {
///         schedule_type: ScheduleType::Once,
///         due_date: None,
///         lead_time: 0,
///         last_done: None,
///     },
///     repeat_config: None,
///     is_sequential: false,
///     is_acknowledged: false,
///     last_completed_at: None,
/// };
///
/// let child_task = PersistedTask {
///     id: child_id.clone(),
///     parent_id: Some(parent_id.clone()),
///     ..parent_task.clone()
/// };
///
/// let mut tasks = HashMap::new();
/// tasks.insert(parent_id.clone(), parent_task);
/// tasks.insert(child_id.clone(), child_task);
///
/// let state = TunnelState {
///     tasks,
///     root_task_ids: vec![parent_id.clone()],
///     places: HashMap::new(),
///     metadata: None,
/// };
///
/// let descendants = get_descendant_ids(&state, &parent_id);
/// assert_eq!(descendants.len(), 1);
/// assert!(descendants.contains(&child_id));
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
///
/// Iteratively walks up the parent chain of the specified `task_id` in `state`.
/// It stops when it reaches a root task (a task with no parent) or a task that cannot
/// be found in the task map. Returns a [`Vec`] containing the IDs of all ancestors.
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::hierarchy::get_ancestor_ids;
/// use tasklens_core::types::{TaskID, TunnelState, PersistedTask, TaskStatus, Schedule, ScheduleType};
/// use std::collections::HashMap;
///
/// let parent_id = TaskID::from("parent");
/// let child_id = TaskID::from("child");
///
/// let parent_task = PersistedTask {
///     id: parent_id.clone(),
///     title: "Parent".to_string(),
///     notes: String::new(),
///     parent_id: None,
///     child_task_ids: vec![child_id.clone()],
///     place_id: None,
///     status: TaskStatus::Pending,
///     importance: 1.0,
///     credit_increment: None,
///     credits: 0.0,
///     desired_credits: 0.0,
///     credits_timestamp: 0,
///     priority_timestamp: 0,
///     schedule: Schedule {
///         schedule_type: ScheduleType::Once,
///         due_date: None,
///         lead_time: 0,
///         last_done: None,
///     },
///     repeat_config: None,
///     is_sequential: false,
///     is_acknowledged: false,
///     last_completed_at: None,
/// };
///
/// let child_task = PersistedTask {
///     id: child_id.clone(),
///     parent_id: Some(parent_id.clone()),
///     ..parent_task.clone()
/// };
///
/// let mut tasks = HashMap::new();
/// tasks.insert(parent_id.clone(), parent_task);
/// tasks.insert(child_id.clone(), child_task);
///
/// let state = TunnelState {
///     tasks,
///     root_task_ids: vec![parent_id.clone()],
///     places: HashMap::new(),
///     metadata: None,
/// };
///
/// let ancestors = get_ancestor_ids(&state, &child_id);
/// assert_eq!(ancestors.len(), 1);
/// assert_eq!(ancestors[0], parent_id);
/// ```
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
/// Searches for the specified `task_id` in its parent's `child_task_ids` array, or in
/// `state.root_task_ids` if it has no parent. Returns the ID of the sibling that immediately
/// precedes it. Returns [`None`] if the task is the first child, or if the task or its parent
/// does not exist.
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::hierarchy::get_previous_sibling;
/// use tasklens_core::types::{TaskID, TunnelState, PersistedTask, TaskStatus, Schedule, ScheduleType};
/// use std::collections::HashMap;
///
/// let parent_id = TaskID::from("parent");
/// let child1_id = TaskID::from("child1");
/// let child2_id = TaskID::from("child2");
///
/// let parent_task = PersistedTask {
///     id: parent_id.clone(),
///     title: "Parent".to_string(),
///     notes: String::new(),
///     parent_id: None,
///     child_task_ids: vec![child1_id.clone(), child2_id.clone()],
///     place_id: None,
///     status: TaskStatus::Pending,
///     importance: 1.0,
///     credit_increment: None,
///     credits: 0.0,
///     desired_credits: 0.0,
///     credits_timestamp: 0,
///     priority_timestamp: 0,
///     schedule: Schedule {
///         schedule_type: ScheduleType::Once,
///         due_date: None,
///         lead_time: 0,
///         last_done: None,
///     },
///     repeat_config: None,
///     is_sequential: false,
///     is_acknowledged: false,
///     last_completed_at: None,
/// };
///
/// let child1_task = PersistedTask {
///     id: child1_id.clone(),
///     parent_id: Some(parent_id.clone()),
///     ..parent_task.clone()
/// };
///
/// let child2_task = PersistedTask {
///     id: child2_id.clone(),
///     parent_id: Some(parent_id.clone()),
///     ..parent_task.clone()
/// };
///
/// let mut tasks = HashMap::new();
/// tasks.insert(parent_id.clone(), parent_task);
/// tasks.insert(child1_id.clone(), child1_task);
/// tasks.insert(child2_id.clone(), child2_task);
///
/// let state = TunnelState {
///     tasks,
///     root_task_ids: vec![parent_id.clone()],
///     places: HashMap::new(),
///     metadata: None,
/// };
///
/// let previous = get_previous_sibling(&state, &child2_id);
/// assert_eq!(previous, Some(child1_id));
/// ```
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
