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
 * Converts milliseconds to days.
 * @param ms Number of milliseconds.
 */
export function millisecondsToDays(ms: number): number {
  return ms / (24 * 60 * 60 * 1000);
}
