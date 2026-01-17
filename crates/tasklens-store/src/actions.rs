use serde::{Deserialize, Serialize};
use tasklens_core::types::{PlaceID, TaskID, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    CreateTask {
        id: TaskID,
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
        current_time: i64,
    },
    MoveTask {
        id: TaskID,
        new_parent_id: Option<TaskID>,
    },
    RefreshLifecycle {
        current_time: i64,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdates {
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    /// Optional update: `Some(None)` clears the place, `Some(Some(id))` sets it, `None` keeps current.
    pub place_id: Option<Option<PlaceID>>,
    /// Optional update: `Some(None)` clears the due date, `Some(Some(ts))` sets it, `None` keeps current.
    pub due_date: Option<Option<i64>>,
    pub schedule_type: Option<tasklens_core::types::ScheduleType>,
    pub lead_time: Option<Option<i64>>,
    pub repeat_config: Option<Option<tasklens_core::types::RepeatConfig>>,
    pub is_sequential: Option<bool>,
}
