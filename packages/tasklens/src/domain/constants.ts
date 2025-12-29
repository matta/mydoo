/**
 * Half-life for credit decay calculation (7 days in milliseconds).
 * Credits decay exponentially with this half-life to prioritize recent work.
 */
export const CREDITS_HALF_LIFE_MILLIS = 7 * 24 * 60 * 60 * 1000;
