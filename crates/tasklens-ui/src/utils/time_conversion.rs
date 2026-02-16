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
