use crate::types::{TaskID, TaskStatus, TunnelState};

/// Half-life for credit decay in milliseconds (7 days).
const CREDITS_HALF_LIFE_MS: f64 = 604_800_000.0;

/// Complete a task and attribute credits per algorithm.md §5.1.
///
/// This function:
/// 1. Applies exponential decay to the task's existing credits based on time elapsed
/// 2. Adds the task's credit_increment to its credits
/// 3. Updates credits_timestamp to current_time
///
/// Note: This only modifies the completed task's stored credits. Ancestors are NOT modified.
/// Parent effective_credits aggregation happens at query time via get_prioritized_tasks.
pub fn complete_task(state: &mut TunnelState, task_id: &TaskID, current_time: i64) {
    if let Some(task) = state.tasks.get_mut(task_id) {
        // Step 1: Apply decay to existing credits (bring history to present)
        let time_delta_ms = (current_time - task.credits_timestamp) as f64;
        let decay_factor = 0.5_f64.powf(time_delta_ms / CREDITS_HALF_LIFE_MS);
        let decayed_credits = task.credits * decay_factor;

        // Step 2: Add credit_increment (default 0.5 per algorithm.md §4.1)
        let increment = task.credit_increment.unwrap_or(0.5);
        task.credits = decayed_credits + increment;

        // Step 3: Checkpoint time
        task.credits_timestamp = current_time;
    }
}

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

    fn make_test_task(id: &str, credits: f64, credit_increment: Option<f64>) -> PersistedTask {
        PersistedTask {
            id: TaskID::from(id),
            title: format!("Task {}", id),
            notes: "".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            status: TaskStatus::Pending,
            importance: 1.0,
            credit_increment,
            credits,
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
        }
    }

    #[test]
    fn test_complete_task_adds_credit_increment() {
        let task_id = TaskID::from("task-1");
        let mut tasks = HashMap::new();
        tasks.insert(task_id.clone(), make_test_task("task-1", 0.0, Some(0.5)));

        let mut state = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            metadata: None,
        };

        complete_task(&mut state, &task_id, 0);

        let task = &state.tasks[&task_id];
        assert!((task.credits - 0.5).abs() < 0.001, "Credits should be 0.5");
        assert_eq!(task.credits_timestamp, 0);
    }

    #[test]
    fn test_complete_task_applies_decay_before_adding() {
        let task_id = TaskID::from("task-1");
        let mut task = make_test_task("task-1", 100.0, Some(0.5));
        task.credits_timestamp = 0;

        let mut tasks = HashMap::new();
        tasks.insert(task_id.clone(), task);

        let mut state = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            metadata: None,
        };

        // After 7 days (one half-life), 100 credits decay to 50, then +0.5 = 50.5
        let seven_days_ms: i64 = 604_800_000;
        complete_task(&mut state, &task_id, seven_days_ms);

        let task = &state.tasks[&task_id];
        assert!(
            (task.credits - 50.5).abs() < 0.001,
            "Credits should be 50.5, got {}",
            task.credits
        );
        assert_eq!(task.credits_timestamp, seven_days_ms);
    }

    #[test]
    fn test_complete_task_cumulative() {
        let task_id = TaskID::from("task-1");
        let mut tasks = HashMap::new();
        tasks.insert(task_id.clone(), make_test_task("task-1", 0.0, Some(1.0)));

        let mut state = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            metadata: None,
        };

        complete_task(&mut state, &task_id, 0);
        assert!((state.tasks[&task_id].credits - 1.0).abs() < 0.001);

        complete_task(&mut state, &task_id, 0);
        assert!((state.tasks[&task_id].credits - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_complete_task_uses_default_increment() {
        let task_id = TaskID::from("task-1");
        let mut tasks = HashMap::new();
        tasks.insert(task_id.clone(), make_test_task("task-1", 0.0, None));

        let mut state = TunnelState {
            next_task_id: 1,
            next_place_id: 1,
            tasks,
            root_task_ids: vec![task_id.clone()],
            places: HashMap::new(),
            metadata: None,
        };

        complete_task(&mut state, &task_id, 0);

        // Default credit_increment is 0.5 per algorithm.md §4.1
        let task = &state.tasks[&task_id];
        assert!((task.credits - 0.5).abs() < 0.001, "Credits should be 0.5");
    }
}
