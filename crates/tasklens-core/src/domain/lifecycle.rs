use crate::types::{TaskStatus, TunnelState};

/// Acknowledge all tasks that are currently "Done".
///
/// This moves them from "Done" to "Acknowledged" status effectively,
/// though they stay as "Done" in the status field, and `is_acknowledged` is set to true.
pub fn acknowledge_completed_tasks(state: &mut TunnelState) {
    for task in state.tasks.values_mut() {
        if task.status == TaskStatus::Done && !task.is_acknowledged {
            task.is_acknowledged = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PersistedTask, Schedule, ScheduleType, TaskID};
    use std::collections::HashMap;

    #[test]
    fn test_acknowledge_completed_tasks() {
        let task_id = TaskID::from("task-1");
        let mut tasks = HashMap::new();
        tasks.insert(
            task_id.clone(),
            PersistedTask {
                id: task_id.clone(),
                title: "Test Task".to_string(),
                notes: "".to_string(),
                parent_id: None,
                child_task_ids: vec![],
                place_id: None,
                status: TaskStatus::Done,
                importance: 1.0,
                credit_increment: None,
                credits: 0.0,
                desired_credits: 0.0,
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
                last_completed_at: Some(100),
            },
        );

        let mut state = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            metadata: None,
        };

        acknowledge_completed_tasks(&mut state);
        assert!(state.tasks[&task_id].is_acknowledged);
    }
}
