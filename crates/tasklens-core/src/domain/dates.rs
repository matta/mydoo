use crate::types::UrgencyStatus;
use chrono::{DateTime, Datelike};

/// The threshold ratio for urgency transitions.
/// - "Upcoming" begins at `lead_time + (lead_time * URGENCY_THRESHOLD_RATIO)` before due.
/// - "Urgent" begins at `lead_time * URGENCY_THRESHOLD_RATIO` before due.
const URGENCY_THRESHOLD_RATIO: f64 = 0.25;

/// Determines the urgency status of a task based on its effective due date and lead time.
///
/// # Arguments
/// * `effective_due_date` - The timestamp when the task is due (ms).
/// * `effective_lead_time` - The duration (ms) before due date when task becomes active.
/// * `current_time` - The current timestamp (ms).
pub fn get_urgency_status(
    effective_due_date: Option<u64>,
    effective_lead_time: Option<u64>,
    current_time: u64,
) -> UrgencyStatus {
    let (due_date, lead_time) = match (effective_due_date, effective_lead_time) {
        (Some(d), Some(l)) => (d, l),
        _ => return UrgencyStatus::None,
    };

    if current_time > due_date {
        if is_same_day_utc(due_date, current_time).unwrap_or(false) {
            return UrgencyStatus::Urgent;
        }
        return UrgencyStatus::Overdue;
    }

    if is_same_day_utc(due_date, current_time).unwrap_or(false) {
        return UrgencyStatus::Urgent;
    }

    let time_buffer = due_date - current_time;

    // Check "Upcoming": within URGENCY_THRESHOLD_RATIO of lead time BEFORE the window starts.
    if time_buffer > lead_time {
        let upcoming_threshold = lead_time + (lead_time as f64 * URGENCY_THRESHOLD_RATIO) as u64;
        if time_buffer <= upcoming_threshold {
            return UrgencyStatus::Upcoming;
        }
        return UrgencyStatus::None;
    }

    // Urgent: Final URGENCY_THRESHOLD_RATIO of its lead time window
    if time_buffer <= (lead_time as f64 * URGENCY_THRESHOLD_RATIO) as u64 {
        return UrgencyStatus::Urgent;
    }

    UrgencyStatus::Active
}

/// Checks if two timestamps represent the same day in UTC.
///
/// Returns `None` if either timestamp is invalid (out of range).
pub fn is_same_day_utc(t1: u64, t2: u64) -> Option<bool> {
    let dt1 = DateTime::from_timestamp((t1 / 1000) as i64, ((t1 % 1000) * 1_000_000) as u32)?;
    let dt2 = DateTime::from_timestamp((t2 / 1000) as i64, ((t2 % 1000) * 1_000_000) as u32)?;

    Some(dt1.year() == dt2.year() && dt1.month() == dt2.month() && dt1.day() == dt2.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_same_day_utc() {
        // 2026-01-13T01:00:00Z (Monday)
        let t1 = 1736730000000u64;
        // 2026-01-13T03:46:40Z (same day)
        let t2 = 1736740000000u64;
        // 2026-01-14T01:00:00Z (next day)
        let t3 = 1736816400000u64;
        assert!(is_same_day_utc(t1, t2).unwrap());
        assert!(!is_same_day_utc(t1, t3).unwrap());
    }

    #[test]
    fn test_get_urgency_status_overdue() {
        let due_date = 1000u64;
        let lead_time = 100u64;
        let current_time = 1100u64;
        assert_eq!(
            get_urgency_status(Some(due_date), Some(lead_time), current_time),
            UrgencyStatus::Urgent
        );

        let day_ms = 24 * 60 * 60 * 1000u64;
        assert_eq!(
            get_urgency_status(Some(due_date), Some(lead_time), current_time + day_ms),
            UrgencyStatus::Overdue
        );
    }

    #[test]
    fn test_get_urgency_status_upcoming() {
        // 2026-01-14T01:00:00Z
        let due_date = 1736816400000u64;
        // 24 hours lead time
        let lead_time = 24 * 60 * 60 * 1000u64;

        // 24.5h before due -> Upcoming (within 25% buffer of 24h = 6h)
        let current_time = due_date - (24 * 60 * 60 * 1000 + 30 * 60 * 1000);
        assert_eq!(
            get_urgency_status(Some(due_date), Some(lead_time), current_time),
            UrgencyStatus::Upcoming
        );
    }

    #[test]
    fn test_get_urgency_status_active() {
        // 2026-01-14T01:00:00Z
        let due_date = 1736816400000u64;
        // 24 hours lead time
        let lead_time = 24 * 60 * 60 * 1000u64;

        // 12h before due -> Active (inside lead time window, but > 25%)
        let current_time = due_date - 12 * 60 * 60 * 1000;
        assert_eq!(
            get_urgency_status(Some(due_date), Some(lead_time), current_time),
            UrgencyStatus::Active
        );
    }

    #[test]
    fn test_get_urgency_status_none() {
        // 2026-01-14T01:00:00Z
        let due_date = 1736816400000u64;
        // 1 second lead time
        let lead_time = 1000u64;
        // 2 hours before due (way outside the lead time window)
        let current_time = due_date - 2 * 60 * 60 * 1000;
        assert_eq!(
            get_urgency_status(Some(due_date), Some(lead_time), current_time),
            UrgencyStatus::None
        );
    }
}
