/// Calculates the Lead Time Factor for a task based on its due date and the current time.
///
/// The Lead Time Factor determines how "ready" or "urgent" a task is relative to its due date.
/// - 0.0: The task is "Too Early" (hidden).
/// - 0.0 -> 1.0: The task is ramping up in urgency.
/// - 1.0+: The task is fully active.
///
/// Logic matches pass5LeadTimeRamp:
/// - If no due date, the factor is 1.0.
/// - Ramp starts at 2 * lead_time remaining.
/// - Fully ramped at 1 * lead_time remaining.
pub fn calculate_lead_time_factor(due_date: Option<u64>, lead_time: u64, current_time: u64) -> f64 {
    let due_date = match due_date {
        Some(d) => d,
        None => return 1.0,
    };

    if due_date <= current_time {
        return 1.0;
    }

    let time_remaining = due_date - current_time;

    if time_remaining > 2 * lead_time {
        return 0.0;
    }

    let raw_factor = ((2 * lead_time) as f64 - time_remaining as f64) / lead_time as f64;
    raw_factor.clamp(0.0, 1.0)
}

/// Determines if a task is "Ready" to be shown in the UI.
pub fn is_task_ready(due_date: Option<u64>, lead_time: u64, current_time: u64) -> bool {
    match due_date {
        None => true,
        Some(d) => {
            if d <= current_time {
                true
            } else {
                let time_remaining = d - current_time;
                time_remaining <= 2 * lead_time
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_lead_time_factor() {
        let lead_time = 1000;
        let due_date = 5000;

        // Too early (> 2 * lead_time)
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 2000),
            0.0
        );

        // At the start of the ramp (exactly 2 * lead_time)
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 3000),
            0.0
        );

        // Middle of the ramp
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 3500),
            0.5
        );

        // Fully ramped (exactly 1 * lead_time)
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 4000),
            1.0
        );

        // Close to due date
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 4500),
            1.0
        );

        // Past due date
        assert_eq!(
            calculate_lead_time_factor(Some(due_date), lead_time, 6000),
            1.0
        );

        // No due date
        assert_eq!(calculate_lead_time_factor(None, lead_time, 6000), 1.0);
    }

    #[test]
    fn test_is_task_ready() {
        let lead_time = 1000;
        let due_date = 5000;

        assert!(!is_task_ready(Some(due_date), lead_time, 2000));
        assert!(is_task_ready(Some(due_date), lead_time, 3000));
        assert!(is_task_ready(Some(due_date), lead_time, 4000));
        assert!(is_task_ready(Some(due_date), lead_time, 6000));
        assert!(is_task_ready(None, lead_time, 6000));
    }
}
