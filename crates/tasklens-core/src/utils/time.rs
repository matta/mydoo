use crate::types::Frequency;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in milliseconds since the Unix Epoch.
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// Helper: Convert frequency/interval to milliseconds.
/// Note: simplified logic for MVP (ignoring leap years/DST nuances for basic "days/weeks").
pub fn get_interval_ms(frequency: Frequency, interval: u32) -> u64 {
    const ONE_MINUTE_MS: u64 = 60 * 1000;
    const ONE_HOUR_MS: u64 = 60 * ONE_MINUTE_MS;
    const ONE_DAY_MS: u64 = 24 * ONE_HOUR_MS;

    match frequency {
        Frequency::Minutes => interval as u64 * ONE_MINUTE_MS,
        Frequency::Hours => interval as u64 * ONE_HOUR_MS,
        Frequency::Daily => interval as u64 * ONE_DAY_MS,
        Frequency::Weekly => interval as u64 * 7 * ONE_DAY_MS,
        Frequency::Monthly => interval as u64 * 30 * ONE_DAY_MS,
        Frequency::Yearly => interval as u64 * 365 * ONE_DAY_MS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interval_ms() {
        assert_eq!(get_interval_ms(Frequency::Minutes, 1), 60_000);
        assert_eq!(get_interval_ms(Frequency::Hours, 1), 3_600_000);
        assert_eq!(get_interval_ms(Frequency::Daily, 1), 86_400_000);
        assert_eq!(get_interval_ms(Frequency::Weekly, 1), 604_800_000);
        assert_eq!(get_interval_ms(Frequency::Monthly, 1), 2_592_000_000);
        assert_eq!(get_interval_ms(Frequency::Yearly, 1), 31_536_000_000);
    }
}
