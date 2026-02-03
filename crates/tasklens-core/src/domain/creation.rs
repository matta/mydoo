//! Task creation factory with inheritance rules.
//!
//! This module provides [`create_new_task`] which initializes new tasks with
//! correct defaults and inherited properties from parents (place, effort).

use crate::domain::constants::{DEFAULT_CREDIT_INCREMENT, DEFAULT_LEAD_TIME_MILLIS};
use crate::types::{PersistedTask, Schedule, ScheduleType, TaskID, TaskStatus};

/// Creates a new task with default values and inheritance from parent.
pub fn create_new_task(id: TaskID, title: String, parent: Option<&PersistedTask>) -> PersistedTask {
    let parent_id = parent.map(|p| p.id.clone());

    // Inheritance Rules
    let mut place_id = None;
    let mut credit_increment = Some(DEFAULT_CREDIT_INCREMENT);

    if let Some(p) = parent {
        place_id = p.place_id.clone();
        credit_increment = p.credit_increment;
    }

    PersistedTask {
        id,
        title,
        notes: String::new(),
        parent_id,
        child_task_ids: Vec::new(),
        place_id,
        status: TaskStatus::Pending,
        importance: 0.5,
        credit_increment,
        credits: 0.0,
        desired_credits: 1.0,
        credits_timestamp: 0,
        priority_timestamp: 0,
        schedule: Schedule {
            schedule_type: ScheduleType::Once,
            due_date: None,
            lead_time: DEFAULT_LEAD_TIME_MILLIS,
            last_done: None,
        },
        repeat_config: None,
        is_sequential: false,
        is_acknowledged: false,
        last_completed_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::constants::DEFAULT_CREDIT_INCREMENT;
    use crate::types::PlaceID;

    #[test]
    fn test_create_new_task_defaults() {
        let task = create_new_task(TaskID::new(), "Root Task".to_string(), None);
        assert_eq!(task.title, "Root Task");
        assert_eq!(task.parent_id, None);
        assert_eq!(task.place_id, None);
        assert_eq!(task.credit_increment, Some(DEFAULT_CREDIT_INCREMENT));
        assert_eq!(task.schedule.lead_time, DEFAULT_LEAD_TIME_MILLIS);
    }

    #[test]
    fn test_create_new_task_inheritance() {
        let mut parent = create_new_task(TaskID::new(), "Parent".to_string(), None);
        parent.place_id = Some(PlaceID::from("Work"));
        parent.credit_increment = Some(2.0);

        let child = create_new_task(TaskID::new(), "Child".to_string(), Some(&parent));

        // Inherited
        assert_eq!(child.place_id, parent.place_id);
        assert_eq!(child.credit_increment, parent.credit_increment);
        assert_eq!(child.parent_id, Some(parent.id));

        // Default overrides
        assert_eq!(child.schedule.lead_time, DEFAULT_LEAD_TIME_MILLIS);
    }
}
