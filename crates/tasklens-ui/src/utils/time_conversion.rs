//! Utility functions for time and period conversion in the UI.

/// Milliseconds in one minute.
pub(crate) const MINUTE_MS: i64 = 60 * 1000;
/// Milliseconds in one hour.
pub(crate) const HOUR_MS: i64 = 60 * MINUTE_MS;
/// Milliseconds in one day.
pub(crate) const DAY_MS: i64 = 24 * HOUR_MS;
/// Milliseconds in one week.
pub(crate) const WEEK_MS: i64 = 7 * DAY_MS;

/// Converts milliseconds to a human-readable period (e.g., number of days).
///
/// Returns (value, unit). Automatically chooses best unit (Days, Hours, or Minutes).
pub(crate) fn ms_to_period(ms: i64) -> (u32, String) {
    if ms >= DAY_MS && ms % DAY_MS == 0 {
        ((ms / DAY_MS) as u32, "Days".to_string())
    } else if ms >= HOUR_MS && ms % HOUR_MS == 0 {
        ((ms / HOUR_MS) as u32, "Hours".to_string())
    } else {
        ((ms / MINUTE_MS) as u32, "Minutes".to_string())
    }
}

/// Converts a period (value + unit) to milliseconds.
///
/// Supports "Minutes", "Hours", "Days", "Weeks", "Months", "Years" (and lowercase/plural variations).
/// Unrecognized units default to days with a warning.
pub(crate) fn period_to_ms(value: u32, unit: &str) -> i64 {
    match unit.to_lowercase().as_str() {
        "minutes" | "minute" | "mins" | "min" => value as i64 * MINUTE_MS,
        "hours" | "hour" | "hrs" | "hr" => value as i64 * HOUR_MS,
        "days" | "day" => value as i64 * DAY_MS,
        "weeks" | "week" => value as i64 * WEEK_MS,
        "months" | "month" => value as i64 * DAY_MS * 30,
        "years" | "year" => value as i64 * DAY_MS * 365,
        _ => {
            tracing::warn!("Unrecognized time unit '{}', defaulting to days", unit);
            value as i64 * DAY_MS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ms_to_period_multiples() {
        // Multiples of Days
        assert_eq!(ms_to_period(DAY_MS), (1, "Days".to_string()));
        assert_eq!(ms_to_period(2 * DAY_MS), (2, "Days".to_string()));
        assert_eq!(ms_to_period(7 * DAY_MS), (7, "Days".to_string()));

        // Multiples of Hours (but not Days)
        assert_eq!(ms_to_period(HOUR_MS), (1, "Hours".to_string()));
        assert_eq!(ms_to_period(2 * HOUR_MS), (2, "Hours".to_string()));
        assert_eq!(ms_to_period(23 * HOUR_MS), (23, "Hours".to_string()));
        assert_eq!(ms_to_period(25 * HOUR_MS), (25, "Hours".to_string()));

        // Multiples of Minutes (but not Hours)
        assert_eq!(ms_to_period(MINUTE_MS), (1, "Minutes".to_string()));
        assert_eq!(ms_to_period(59 * MINUTE_MS), (59, "Minutes".to_string()));
        assert_eq!(ms_to_period(61 * MINUTE_MS), (61, "Minutes".to_string()));
    }

    #[test]
    fn test_ms_to_period_edge_cases() {
        // Edge cases and non-multiples
        assert_eq!(ms_to_period(0), (0, "Minutes".to_string()));
        assert_eq!(ms_to_period(30 * 1000), (0, "Minutes".to_string())); // 30 seconds
        assert_eq!(ms_to_period(MINUTE_MS + 1), (1, "Minutes".to_string()));
        assert_eq!(ms_to_period(HOUR_MS + 1), (60, "Minutes".to_string()));
        assert_eq!(ms_to_period(DAY_MS + 1), (1440, "Minutes".to_string()));
    }

    #[test]
    fn test_period_to_ms_minutes_hours() {
        // Minutes
        assert_eq!(period_to_ms(1, "minutes"), MINUTE_MS);
        assert_eq!(period_to_ms(2, "minute"), 2 * MINUTE_MS);
        assert_eq!(period_to_ms(3, "mins"), 3 * MINUTE_MS);
        assert_eq!(period_to_ms(4, "min"), 4 * MINUTE_MS);

        // Hours
        assert_eq!(period_to_ms(1, "hours"), HOUR_MS);
        assert_eq!(period_to_ms(2, "hour"), 2 * HOUR_MS);
        assert_eq!(period_to_ms(3, "hrs"), 3 * HOUR_MS);
        assert_eq!(period_to_ms(4, "hr"), 4 * HOUR_MS);
    }

    #[test]
    fn test_period_to_ms_days_weeks_months_years() {
        // Days
        assert_eq!(period_to_ms(1, "days"), DAY_MS);
        assert_eq!(period_to_ms(2, "day"), 2 * DAY_MS);

        // Weeks
        assert_eq!(period_to_ms(1, "weeks"), WEEK_MS);
        assert_eq!(period_to_ms(2, "week"), 2 * WEEK_MS);

        // Months
        assert_eq!(period_to_ms(1, "months"), DAY_MS * 30);
        assert_eq!(period_to_ms(2, "month"), 2 * DAY_MS * 30);

        // Years
        assert_eq!(period_to_ms(1, "years"), DAY_MS * 365);
        assert_eq!(period_to_ms(2, "year"), 2 * DAY_MS * 365);
    }

    #[test]
    fn test_period_to_ms_misc() {
        // Case insensitivity
        assert_eq!(period_to_ms(1, "DAYS"), DAY_MS);
        assert_eq!(period_to_ms(1, "MinUTes"), MINUTE_MS);

        // Unknown unit defaults to days
        assert_eq!(period_to_ms(5, "unknown"), 5 * DAY_MS);
    }
}
