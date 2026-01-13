/// Half-life for credit decay calculation (7 days in milliseconds).
pub const CREDITS_HALF_LIFE_MILLIS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Default lead time in milliseconds (1 week).
/// Note: Compliance tests assume 1 week ramp.
pub const DEFAULT_TASK_LEAD_TIME_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Default credit increment for tasks without explicit assignment.
pub const DEFAULT_CREDIT_INCREMENT: f64 = 0.5;

/// Sensitivity for feedback calculation.
pub const FEEDBACK_SENSITIVITY: f64 = 2.0;

/// Minimum priority threshold to be considered "visible" in the list.
pub const MIN_PRIORITY: f64 = 0.001;
