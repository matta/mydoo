use serde::{Deserialize, Serialize};

use crate::types::{PlaceID, RepeatConfig, ScheduleType, TaskID, TaskStatus};

/// Represents an atomic operation that mutates the application state.
///
/// This enum encapsulates all valid domain actions, such as creating, updating, or deleting
/// tasks and places. These actions are processed by the `run_action` function, which applies
/// them to the Automerge document while enforcing business logic and validation rules.
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::actions::Action;
/// use tasklens_core::types::TaskID;
///
/// let id = TaskID::new();
/// let action = Action::CreateTask {
///     id,
///     parent_id: None,
///     title: "New Task".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Creates a new task with the specified details.
    CreateTask {
        /// The unique identifier for the new task.
        id: TaskID,
        /// The parent task's identifier, if any.
        parent_id: Option<TaskID>,
        /// The title of the task.
        title: String,
    },
    /// Updates an existing task with partial changes.
    UpdateTask {
        /// The identifier of the task to update.
        id: TaskID,
        /// The set of fields to update.
        updates: TaskUpdates,
    },
    /// Permanently deletes a task and its descendants.
    DeleteTask {
        /// The identifier of the task to delete.
        id: TaskID,
    },
    /// Marks a task as complete and applies credit decay logic.
    CompleteTask {
        /// The identifier of the task to complete.
        id: TaskID,
        /// The timestamp of completion (Unix milliseconds).
        current_time: i64,
    },
    /// Moves a task to a new parent or to the root level.
    MoveTask {
        /// The identifier of the task to move.
        id: TaskID,
        /// The new parent's identifier, or `None` to move to root.
        new_parent_id: Option<TaskID>,
    },
    /// Refreshes lifecycle states, such as waking up routine tasks.
    RefreshLifecycle {
        /// The current timestamp (Unix milliseconds).
        current_time: i64,
    },
    /// Updates the desired credit distribution across tasks.
    SetBalanceDistribution {
        /// A map of task IDs to their new desired credit values.
        distribution: std::collections::HashMap<TaskID, f64>,
    },
    /// Creates a new place (context).
    CreatePlace {
        /// The unique identifier for the new place.
        id: PlaceID,
        /// The display name of the place.
        name: String,
        /// The operating hours configuration as a JSON string.
        hours: String,
        /// A list of places included within this place.
        included_places: Vec<PlaceID>,
    },
    /// Updates an existing place with partial changes.
    UpdatePlace {
        /// The identifier of the place to update.
        id: PlaceID,
        /// The set of fields to update.
        updates: PlaceUpdates,
    },
    /// Deletes a place.
    DeletePlace {
        /// The identifier of the place to delete.
        id: PlaceID,
    },
}

/// Partial updates for a Place.
///
/// Each `Some` field overwrites the corresponding value; `None` fields are left unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaceUpdates {
    pub name: Option<String>,
    pub hours: Option<String>,
    pub included_places: Option<Vec<PlaceID>>,
}

/// Represents partial updates to be applied to a task.
///
/// Fields wrapped in `Option` represent values that can be updated.
/// - `None`: The field remains unchanged.
/// - `Some(value)`: The field is updated to `value`.
///
/// For nullable fields in the domain (like `place_id`), `Option<Option<T>>` is used:
/// - `None`: No change.
/// - `Some(Some(v))`: Set to `v`.
/// - `Some(None)`: Clear the value (set to `null` or `None`).
///
/// # Examples
///
/// ```
/// use tasklens_core::domain::actions::TaskUpdates;
/// use tasklens_core::types::PlaceID;
///
/// // Update title and clear the place_id
/// let updates = TaskUpdates {
///     title: Some("New Title".to_string()),
///     place_id: Some(None),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdates {
    /// Updates the task's title.
    pub title: Option<String>,
    /// Updates the task's notes.
    pub notes: Option<String>,
    /// Updates the task's status (e.g., Pending, Done).
    pub status: Option<TaskStatus>,
    /// Updates or clears the assigned place.
    ///
    /// * `Some(Some(id))` - Assigns the task to the specified place.
    /// * `Some(None)` - Removes the task from any place.
    /// * `None` - Leaves the current place assignment unchanged.
    pub place_id: Option<Option<PlaceID>>,
    /// Updates or clears the due date.
    ///
    /// * `Some(Some(ts))` - Sets the due date (Unix timestamp in milliseconds).
    /// * `Some(None)` - Removes the due date.
    /// * `None` - Leaves the due date unchanged.
    pub due_date: Option<Option<i64>>,
    /// Updates the task's scheduling type.
    pub schedule_type: Option<ScheduleType>,
    /// Updates the lead time (in milliseconds) before the due date.
    pub lead_time: Option<i64>,
    /// Updates or clears the repeat configuration.
    pub repeat_config: Option<Option<RepeatConfig>>,
    /// Updates whether the task's children are sequential.
    pub is_sequential: Option<bool>,
    /// Updates the current credit balance.
    pub credits: Option<f64>,
    /// Updates the target credit goal.
    pub desired_credits: Option<f64>,
    /// Updates the credit increment earned upon completion.
    pub credit_increment: Option<f64>,
    /// Updates the base importance value.
    pub importance: Option<f64>,
    /// Updates the acknowledgement status (for completed tasks).
    pub is_acknowledged: Option<bool>,
    /// Updates or clears the timestamp of the last completion.
    pub last_done: Option<Option<i64>>,
    /// Updates the timestamp when credits were last decayed/calculated.
    pub credits_timestamp: Option<i64>,
    /// Updates the timestamp when priority was last calculated.
    pub priority_timestamp: Option<i64>,
}
