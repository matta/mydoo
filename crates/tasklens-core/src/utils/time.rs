use crate::types::Frequency;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in milliseconds since the Unix Epoch.
pub fn get_current_timestamp() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as f64
    }
}

pub const DEFAULT_TASK_LEAD_TIME_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Helper: Convert frequency/interval to milliseconds.
/// Note: simplified logic for MVP (ignoring leap years/DST nuances for basic "days/weeks").
pub fn get_interval_ms(frequency: Frequency, interval: f64) -> f64 {
    const ONE_MINUTE_MS: f64 = 60.0 * 1000.0;
    const ONE_HOUR_MS: f64 = 60.0 * ONE_MINUTE_MS;
    const ONE_DAY_MS: f64 = 24.0 * ONE_HOUR_MS;

    match frequency {
        Frequency::Minutes => interval * ONE_MINUTE_MS,
        Frequency::Hours => interval * ONE_HOUR_MS,
        Frequency::Daily => interval * ONE_DAY_MS,
        Frequency::Weekly => interval * 7.0 * ONE_DAY_MS,
        Frequency::Monthly => interval * 30.0 * ONE_DAY_MS,
        Frequency::Yearly => interval * 365.0 * ONE_DAY_MS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interval_ms() {
        assert_eq!(get_interval_ms(Frequency::Minutes, 1.0), 60_000.0);
        assert_eq!(get_interval_ms(Frequency::Hours, 1.0), 3_600_000.0);
        assert_eq!(get_interval_ms(Frequency::Daily, 1.0), 86_400_000.0);
        assert_eq!(get_interval_ms(Frequency::Weekly, 1.0), 604_800_000.0);
        assert_eq!(get_interval_ms(Frequency::Monthly, 1.0), 2_592_000_000.0);
        assert_eq!(get_interval_ms(Frequency::Yearly, 1.0), 31_536_000_000.0);
    }
}
