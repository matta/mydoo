use crate::domain::constants::{
    FEEDBACK_DEVIATION_RATIO_CAP, FEEDBACK_EPSILON, FEEDBACK_SENSITIVITY,
};
use crate::types::EnrichedTask;

/// Aggregated feedback totals for all root tasks.
#[derive(Clone, Copy, Debug)]
pub struct FeedbackTotals {
    /// Sum of desired credits across root tasks.
    pub total_desired_credits: f64,
    /// Sum of effective credits across root tasks.
    pub total_effective_credits: f64,
}

/// Intermediate feedback calculations for a single root task.
#[derive(Clone, Copy, Debug)]
pub struct FeedbackComputation {
    /// Target share of credits for the root task.
    pub target_percent: f64,
    /// Actual share of credits for the root task.
    pub actual_percent: f64,
    /// Deviation ratio capped for stability.
    pub deviation_ratio: f64,
    /// Final feedback factor applied to priority.
    pub feedback_factor: f64,
}

/// Computes feedback totals across all root tasks.
pub fn compute_feedback_totals(tasks: &[EnrichedTask]) -> FeedbackTotals {
    let mut total_desired_credits = 0.0;
    let mut total_effective_credits = 0.0;

    for task in tasks {
        if task.parent_id.is_none() {
            total_desired_credits += task.desired_credits;
            total_effective_credits += task.effective_credits;
        }
    }

    FeedbackTotals {
        total_desired_credits,
        total_effective_credits,
    }
}

/// Computes feedback metrics for a root task given aggregate totals.
pub fn compute_feedback_metrics(
    root: &EnrichedTask,
    totals: FeedbackTotals,
) -> FeedbackComputation {
    if totals.total_desired_credits == 0.0 {
        return FeedbackComputation {
            target_percent: 0.0,
            actual_percent: 0.0,
            deviation_ratio: 1.0,
            feedback_factor: 1.0,
        };
    }

    let target_percent = root.desired_credits / totals.total_desired_credits;
    let effective_denominator = totals
        .total_effective_credits
        .max(FEEDBACK_EPSILON * totals.total_desired_credits);
    let actual_percent = root.effective_credits / effective_denominator;

    let deviation_ratio = if target_percent == 0.0 {
        1.0
    } else {
        target_percent / actual_percent.max(FEEDBACK_EPSILON)
    };
    let deviation_ratio = deviation_ratio.min(FEEDBACK_DEVIATION_RATIO_CAP);

    FeedbackComputation {
        target_percent,
        actual_percent,
        deviation_ratio,
        feedback_factor: deviation_ratio.powf(FEEDBACK_SENSITIVITY),
    }
}

/// Adaptive Feedback Control
/// Calculates how far each Root Goal is from its target allocation.
/// Updates the `feedback_factor` property of each root task.
///
/// Constants: `k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).
pub fn calculate_feedback_factors(tasks: &mut [EnrichedTask]) {
    let totals = compute_feedback_totals(tasks);

    for task in tasks.iter_mut() {
        if task.parent_id.is_none() {
            let computation = compute_feedback_metrics(task, totals);
            task.feedback_factor = computation.feedback_factor;
        }
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

    #[test]
    fn test_compute_feedback_metrics() {
        let tasks = vec![
            mock_root_task("Goal A", 10.0, 5.0),
            mock_root_task("Goal B", 10.0, 10.0),
        ];

        let totals = compute_feedback_totals(&tasks);
        let goal_a = compute_feedback_metrics(&tasks[0], totals);
        let goal_b = compute_feedback_metrics(&tasks[1], totals);

        assert!(goal_a.feedback_factor > 1.0);
        assert!(goal_b.feedback_factor < 1.0);
        assert!(goal_a.target_percent > 0.0);
        assert!(goal_b.actual_percent > 0.0);
    }

    #[test]
    fn test_compute_feedback_metrics_zero_desired() {
        let tasks = vec![mock_root_task("Goal A", 0.0, 0.0)];
        let totals = compute_feedback_totals(&tasks);
        let computation = compute_feedback_metrics(&tasks[0], totals);

        assert_eq!(computation.feedback_factor, 1.0);
        assert_eq!(computation.deviation_ratio, 1.0);
    }
}
