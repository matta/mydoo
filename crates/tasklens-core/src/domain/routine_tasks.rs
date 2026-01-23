use crate::types::{ScheduleType, TaskStatus, TunnelState};
use crate::utils::time::get_interval_ms;

/// Wake up "Routinely" tasks that are due for their next cycle.
///
/// This function checks all "Done" + "Routinely" tasks to see if their wake-up window has arrived.
/// If so, it resets them to "Pending" and updates their due date to the next interval.
pub fn wake_up_routine_tasks(state: &mut TunnelState, current_time: i64) {
    for task in state.tasks.values_mut() {
        if task.status == TaskStatus::Done && task.schedule.schedule_type == ScheduleType::Routinely
        {
            let repeat_config = match &task.repeat_config {
                Some(rc) => rc,
                None => continue, // Safety check: Routinely tasks must have a repeat config
            };

            let last_completed_at = task.last_completed_at.unwrap_or(0);

            // Calculate the next theoretical due date based on completion time + interval
            let interval_ms = get_interval_ms(repeat_config.frequency, repeat_config.interval);
            let next_due_date = last_completed_at + interval_ms;

            // Lead Time defines how early the task appears before it's due
            let lead_time = task.schedule.lead_time;
            let wake_up_time = next_due_date - lead_time;

            if current_time >= wake_up_time {
                // Wake up the task!
                task.status = TaskStatus::Pending;
                task.is_acknowledged = false;

                // Update the schedule for the new cycle
                task.schedule.last_done = Some(last_completed_at);

                // We can clear the explicit due date since priority calculation will derive it from last_done
                task.schedule.due_date = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Frequency, PersistedTask, RepeatConfig, Schedule, TaskID};
    use std::collections::HashMap;

    #[test]
    fn test_wake_up_routine_tasks() {
        let task_id = TaskID::from("routine-1");
        let mut tasks = HashMap::new();
        tasks.insert(
            task_id.clone(),
            PersistedTask {
                id: task_id.clone(),
                title: "Routine Task".to_string(),
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
                    schedule_type: ScheduleType::Routinely,
                    due_date: None,
                    lead_time: 1000,
                    last_done: None,
                },
                repeat_config: Some(RepeatConfig {
                    frequency: Frequency::Daily,
                    interval: 1,
                }),
                is_sequential: false,
                is_acknowledged: true,
                last_completed_at: Some(100000),
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

        // Current time: 100500 ms
        // Next due: 100000 + (24*60*60*1000) = 86,400,000 + 100,000 = 86,500,000
        // Wake up time: 86,500,000 - 1,000 = 86,499,000
        // 100500 < 86499000, so it shouldn't wake up.
        wake_up_routine_tasks(&mut state, 100500);
        assert_eq!(state.tasks[&task_id].status, TaskStatus::Done);

        // Current time: 86500000 (Exactly at due date)
        wake_up_routine_tasks(&mut state, 86500000);
        assert_eq!(state.tasks[&task_id].status, TaskStatus::Pending);
        assert!(!state.tasks[&task_id].is_acknowledged);
    }
}
