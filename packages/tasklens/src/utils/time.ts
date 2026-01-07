// src/utils/time.ts

let now = Date.now;

/**
 * Returns the current timestamp in milliseconds.
 * Can be mocked for testing purposes.
 */
export function getCurrentTimestamp(): number {
  return now();
}

/**
 * Mocks the current timestamp for testing.
 * @param timestamp The timestamp to return, or a function that returns it.
 */
export function mockCurrentTimestamp(timestamp: number | (() => number)): void {
  if (typeof timestamp === 'number') {
    now = () => timestamp;
  } else {
    now = timestamp;
  }
}

/**
 * Resets the current timestamp mock.
 */
export function resetCurrentTimestampMock(): void {
  now = Date.now;
}

/**
 * Converts days to milliseconds.
 * @param days Number of days.
 */
export function daysToMilliseconds(days: number): number {
  return days * 24 * 60 * 60 * 1000;
}

/**
 * Converts hours to milliseconds.
 * @param hours Number of hours.
 */
export function hoursToMilliseconds(hours: number): number {
  return hours * 60 * 60 * 1000;
}

/**
 * Helper: Convert frequency/interval to milliseconds.
 * Note: simplified logic for MVP (ignoring leap years/DST nuances for basic "days/weeks").
 */
export function getIntervalMs(
  frequency: 'minutes' | 'hours' | 'daily' | 'weekly' | 'monthly' | 'yearly',
  interval: number,
): number {
  const ONE_DAY_MS = 24 * 60 * 60 * 1000;
  const ONE_HOUR_MS = 60 * 60 * 1000;
  const ONE_MINUTE_MS = 60 * 1000;

  switch (frequency) {
    case 'minutes':
      return interval * ONE_MINUTE_MS;
    case 'hours':
      return interval * ONE_HOUR_MS;
    case 'daily':
      return interval * ONE_DAY_MS;
    case 'weekly':
      return interval * 7 * ONE_DAY_MS;
    case 'monthly':
      // Approximation: 30 days
      return interval * 30 * ONE_DAY_MS;
    case 'yearly':
      // Approximation: 365 days
      return interval * 365 * ONE_DAY_MS;
  }
}
