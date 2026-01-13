//! Core domain types for TaskLens.
//!
//! These types are designed to be compatible with the TypeScript
//! `TaskSchema` for Automerge document interchange. They use camelCase
//! JSON serialization to match the existing TypeScript schema.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskID(String);

impl TaskID {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for TaskID {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for TaskID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for TaskID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a place/context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlaceID(String);

impl PlaceID {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for PlaceID {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for PlaceID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for PlaceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The completion status of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is not yet completed.
    Pending,
    /// Task has been completed.
    Done,
}

/// The scheduling strategy for a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleType {
    /// A one-time task with no recurrence.
    Once,
    /// A recurring task based on interval since last completion.
    Routinely,
    /// A task with a specific due date.
    DueDate,
    /// A calendar-based scheduled task.
    Calendar,
}

/// Scheduling configuration for a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// The type of schedule (Once, Routinely, DueDate, Calendar).
    #[serde(rename = "type")]
    pub schedule_type: ScheduleType,
    /// Optional due date as Unix timestamp in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<u64>,
    /// Lead time in milliseconds before due date to start showing urgency.
    pub lead_time: u64,
    /// Timestamp of last completion (for Routinely tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_done: Option<u64>,
}

/// Frequency unit for recurring tasks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Minutes,
    Hours,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Configuration for task repetition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepeatConfig {
    /// The unit of frequency (daily, weekly, etc.).
    pub frequency: Frequency,
    /// The interval multiplier (e.g., 2 for "every 2 weeks").
    pub interval: u32,
}

/// A task as persisted in the Automerge document.
///
/// Uses `extra_fields` with `#[serde(flatten)]` to preserve any
/// unknown fields during roundtrip serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedTask {
    pub id: TaskID,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<TaskID>,
    pub child_task_ids: Vec<TaskID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<PlaceID>,
    pub status: TaskStatus,
    pub importance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_increment: Option<f64>,
    pub credits: f64,
    pub desired_credits: f64,
    pub credits_timestamp: u64,
    pub priority_timestamp: u64,
    pub schedule: Schedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_config: Option<RepeatConfig>,
    pub is_sequential: bool,
    #[serde(default)]
    pub is_acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_completed_at: Option<u64>,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

/// A place/context where tasks can be performed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: PlaceID,
    pub hours: String,
    pub included_places: Vec<PlaceID>,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

/// The root state of a TaskLens document.
///
/// This is the top-level structure serialized to/from Automerge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelState {
    pub tasks: HashMap<TaskID, PersistedTask>,
    pub root_task_ids: Vec<TaskID>,
    pub places: HashMap<PlaceID, Place>,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_serialization() {
        let task = PersistedTask {
            id: TaskID::from("task-1".to_string()),
            title: "Test Task".to_string(),
            notes: "Some notes".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            status: TaskStatus::Pending,
            importance: 0.5,
            credit_increment: Some(1.0),
            credits: 0.0,
            desired_credits: 1.0,
            credits_timestamp: 123456789,
            priority_timestamp: 123456789,
            schedule: Schedule {
                schedule_type: ScheduleType::Once,
                due_date: None,
                lead_time: 0,
                last_done: None,
            },
            repeat_config: None,
            is_sequential: false,
            is_acknowledged: false,
            last_completed_at: None,
            extra_fields: HashMap::new(),
        };

        let serialized = serde_json::to_value(&task).unwrap();
        let expected = json!({
            "id": "task-1",
            "title": "Test Task",
            "notes": "Some notes",
            "childTaskIds": [],
            "status": "Pending",
            "importance": 0.5,
            "creditIncrement": 1.0,
            "credits": 0.0,
            "desiredCredits": 1.0,
            "creditsTimestamp": 123456789,
            "priorityTimestamp": 123456789,
            "schedule": {
                "type": "Once",
                "leadTime": 0
            },
            "isSequential": false,
            "isAcknowledged": false
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_tunnel_state_serialization() {
        let mut tasks = HashMap::new();
        let task_id = TaskID::from("task-1".to_string());
        tasks.insert(
            task_id.clone(),
            PersistedTask {
                id: task_id.clone(),
                title: "Test Task".to_string(),
                notes: "".to_string(),
                parent_id: None,
                child_task_ids: vec![],
                place_id: None,
                status: TaskStatus::Pending,
                importance: 1.0,
                credit_increment: None,
                credits: 0.0,
                desired_credits: 1.0,
                credits_timestamp: 0,
                priority_timestamp: 0,
                schedule: Schedule {
                    schedule_type: ScheduleType::Once,
                    due_date: None,
                    lead_time: 0,
                    last_done: None,
                },
                repeat_config: None,
                is_sequential: false,
                is_acknowledged: false,
                last_completed_at: None,
                extra_fields: HashMap::new(),
            },
        );

        let state = TunnelState {
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            extra_fields: HashMap::new(),
        };

        let serialized = serde_json::to_value(&state).unwrap();
        assert!(serialized.get("tasks").is_some());
        assert!(serialized.get("rootTaskIds").is_some());
        assert!(serialized.get("places").is_some());
    }
}
