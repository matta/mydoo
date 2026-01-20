import { hoursToMilliseconds } from "../utils/time";

/**
 * Half-life for credit decay calculation (7 days in milliseconds).
 * Credits decay exponentially with this half-life to prioritize recent work.
 */
export const CREDITS_HALF_LIFE_MILLIS = 7 * 24 * 60 * 60 * 1000;

/**
 * Default importance for new tasks (1.0 = Max Importance).
 */
export const DEFAULT_TASK_IMPORTANCE = 1.0;

/**
 * Default lead time for new tasks in hours (8 hours).
 */
export const DEFAULT_TASK_LEAD_TIME_HOURS = 8;

/**
 * Default lead time in milliseconds.
 */
export const DEFAULT_TASK_LEAD_TIME_MS = hoursToMilliseconds(
  DEFAULT_TASK_LEAD_TIME_HOURS,
);
