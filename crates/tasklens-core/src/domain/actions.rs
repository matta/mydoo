use serde::{Deserialize, Serialize};

use crate::types::{PlaceID, RepeatConfig, ScheduleType, TaskID, TaskStatus};

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
    SetBalanceDistribution {
        distribution: std::collections::HashMap<TaskID, f64>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdates {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub status: Option<TaskStatus>,
    /// Optional update: `Some(None)` clears the place, `Some(Some(id))` sets it, `None` keeps current.
    pub place_id: Option<Option<PlaceID>>,
    /// Optional update: `Some(None)` clears the due date, `Some(Some(ts))` sets it, `None` keeps current.
    pub due_date: Option<Option<i64>>,
    pub schedule_type: Option<ScheduleType>,
    pub lead_time: Option<i64>,
    pub repeat_config: Option<Option<RepeatConfig>>,
    pub is_sequential: Option<bool>,
    pub credits: Option<f64>,
    pub desired_credits: Option<f64>,
    pub credit_increment: Option<f64>,
    pub importance: Option<f64>,
    pub is_acknowledged: Option<bool>,
    /// Optional update: `Some(None)` clears the last done date, `Some(Some(ts))` sets it, `None` keeps current.
    pub last_done: Option<Option<i64>>,
    pub credits_timestamp: Option<i64>,
}
