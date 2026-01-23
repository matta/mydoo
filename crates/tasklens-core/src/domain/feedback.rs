use crate::types::EnrichedTask;

/// Adaptive Feedback Control
/// Calculates how far each Root Goal is from its target allocation.
/// Updates the `feedback_factor` property of each root task.
///
/// Constants: `k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).
pub fn calculate_feedback_factors(tasks: &mut [EnrichedTask]) {
    let k = 2.0; // Sensitivity
    let epsilon = 0.001; // Division by zero protection

    // Identify root tasks
    // In Rust, we need to collect indices or use a separate way to iterate because of mutable borrowing rules.
    let root_task_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.parent_id.is_none())
        .map(|(i, _)| i)
        .collect();

    let mut total_desired_credits = 0.0;
    let mut total_effective_credits = 0.0;

    for &i in &root_task_indices {
        let root = &tasks[i];
        total_desired_credits += root.desired_credits;
        total_effective_credits += root.effective_credits;
    }

    for &i in &root_task_indices {
        if total_desired_credits == 0.0 {
            tasks[i].feedback_factor = 1.0;
            continue;
        }

        let root = &tasks[i];
        let target_percent = root.desired_credits / total_desired_credits;

        // Ensure ActualPercent is never lower than epsilon for calculation stability
        let actual_percent =
            root.effective_credits / total_effective_credits.max(epsilon * total_desired_credits);

        let deviation_ratio = if target_percent == 0.0 {
            1.0
        } else {
            target_percent / actual_percent.max(epsilon)
        };

        // Cap DeviationRatio to prevent extreme spikes (spec says 1000.0)
        let deviation_ratio = deviation_ratio.min(1000.0);

        tasks[i].feedback_factor = deviation_ratio.powf(k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Schedule, ScheduleType, TaskID, TaskStatus};

    fn mock_root_task(id: &str, desired: f64, effective: f64) -> EnrichedTask {
        EnrichedTask {
            id: TaskID::from(id),
            title: id.to_string(),
            notes: "".to_string(),
            parent_id: None,
            child_task_ids: vec![],
            place_id: None,
            status: TaskStatus::Pending,
            importance: 1.0,
            credit_increment: None,
            credits: 0.0,
            desired_credits: desired,
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
            effective_credits: effective,
            feedback_factor: 1.0,
            lead_time_factor: 1.0,
            normalized_importance: 1.0,
            priority: 0.0,
            visibility: true,
            outline_index: 0.0,
            is_container: false,
            is_pending: true,
            is_ready: true,
            effective_due_date: None,
            effective_lead_time: None,
            effective_schedule_source: None,
        }
    }

    #[test]
    fn test_calculate_feedback_factors() {
        let mut tasks = vec![
            mock_root_task("Goal A", 10.0, 5.0), // 50% target, but currently 33% of effort (if B is 10)
            mock_root_task("Goal B", 10.0, 10.0), // 50% target, but currently 66% of effort
        ];

        calculate_feedback_factors(&mut tasks);

        // Goal A is under-served (target 0.5, actual 0.33), so factor should be > 1.0
        assert!(tasks[0].feedback_factor > 1.0);
        // Goal B is over-served (target 0.5, actual 0.66), so factor should be < 1.0
        assert!(tasks[1].feedback_factor < 1.0);
    }
}
