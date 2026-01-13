use serde::{Deserialize, Serialize};
use tasklens_core::types::{PlaceID, TaskID, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    CreateTask {
        parent_id: Option<TaskID>,
        title: String,
    },
    UpdateTask {
        id: TaskID,
        updates: TaskUpdates,
    },
    DeleteTask {
        id: TaskID,
    },
    CompleteTask {
        id: TaskID,
    },
    MoveTask {
        id: TaskID,
        new_parent_id: Option<TaskID>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdates {
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    /// Optional update: `Some(None)` clears the place, `Some(Some(id))` sets it, `None` keeps current.
    pub place_id: Option<Option<PlaceID>>,
    /// Optional update: `Some(None)` clears the due date, `Some(Some(ts))` sets it, `None` keeps current.
    pub due_date: Option<Option<f64>>,
    // Add more fields as needed based on PersistedTask
}
