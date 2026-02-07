/// Half-life for credit decay calculation (7 days in milliseconds).
pub const CREDITS_HALF_LIFE_MILLIS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Default credit increment for tasks without explicit assignment.
pub const DEFAULT_CREDIT_INCREMENT: f64 = 0.5;

/// Sensitivity for feedback calculation.
pub const FEEDBACK_SENSITIVITY: f64 = 2.0;

/// Epsilon for feedback calculation to avoid division by zero.
pub const FEEDBACK_EPSILON: f64 = 0.001;

/// Cap for deviation ratio to avoid runaway priorities when actual is near zero.
pub const FEEDBACK_DEVIATION_RATIO_CAP: f64 = 1000.0;

/// Minimum priority threshold to be considered "visible" in the list.
pub const MIN_PRIORITY: f64 = 0.001;

/// Default lead time in milliseconds (8 hours).
pub const DEFAULT_LEAD_TIME_MILLIS: i64 = 8 * 60 * 60 * 1000;
