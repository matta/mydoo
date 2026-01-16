//! Utility functions for time and period conversion in the UI.

/// Milliseconds in one minute.
pub const MINUTE_MS: f64 = 60.0 * 1000.0;
/// Milliseconds in one hour.
pub const HOUR_MS: f64 = 60.0 * MINUTE_MS;
/// Milliseconds in one day.
pub const DAY_MS: f64 = 24.0 * HOUR_MS;
/// Milliseconds in one week.
pub const WEEK_MS: f64 = 7.0 * DAY_MS;

/// Default lead time (7 days) in milliseconds.
pub const DEFAULT_LEAD_TIME_MS: f64 = WEEK_MS;

/// Converts milliseconds to a human-readable period (e.g., number of days).
///
/// Returns (value, unit). Automatically chooses best unit (Days, Hours, or Minutes).
pub fn ms_to_period(ms: f64) -> (u32, String) {
    if ms >= DAY_MS && ms % DAY_MS == 0.0 {
        ((ms / DAY_MS).round() as u32, "Days".to_string())
    } else if ms >= HOUR_MS && ms % HOUR_MS == 0.0 {
        ((ms / HOUR_MS).round() as u32, "Hours".to_string())
    } else {
        ((ms / MINUTE_MS).round() as u32, "Minutes".to_string())
    }
}

/// Converts a period (value + unit) to milliseconds.
///
/// Supports "Minutes", "Hours", "Days", "Weeks", "Months", "Years" (and lowercase/plural variations).
/// Unrecognized units default to days with a warning.
pub fn period_to_ms(value: u32, unit: &str) -> f64 {
    match unit.to_lowercase().as_str() {
        "minutes" | "minute" | "mins" | "min" => value as f64 * MINUTE_MS,
        "hours" | "hour" | "hrs" | "hr" => value as f64 * HOUR_MS,
        "days" | "day" => value as f64 * DAY_MS,
        "weeks" | "week" => value as f64 * WEEK_MS,
        "months" | "month" => value as f64 * DAY_MS * 30.0,
        "years" | "year" => value as f64 * DAY_MS * 365.0,
        _ => {
            tracing::warn!("Unrecognized time unit '{}', defaulting to days", unit);
            value as f64 * DAY_MS
        }
    }
}
