/**
 * Calculates the Lead Time Factor for a task based on its due date and the current time.
 *
 * The Lead Time Factor determines how "ready" or "urgent" a task is relative to its due date.
 * - 0.0: The task is "Too Early" (hidden).
 * - 0.0 -> 1.0: The task is ramping up in urgency.
 * - 1.0+: The task is fully active.
 *
 * Logic matches pass5LeadTimeRamp:
 * - If no due date, the factor is 1.0.
 * - Ramp starts at 2 * leadTime remaining.
 * - Fully ramped at 1 * leadTime remaining.
 *
 * @param dueDate - The task's due date timestamp (ms), or undefined if no due date
 * @param leadTime - The lead time in milliseconds
 * @param currentTime - The current timestamp in milliseconds
 */
export function calculateLeadTimeFactor(
  dueDate: number | undefined,
  leadTime: number,
  currentTime: number,
): number {
  if (dueDate === undefined) {
    return 1.0;
  }

  const timeRemaining = dueDate - currentTime;

  if (timeRemaining > 2 * leadTime) {
    return 0.0;
  }

  const rawFactor = (2 * leadTime - timeRemaining) / leadTime;
  return Math.max(0, Math.min(1, rawFactor));
}

/**
 * Determines if a task is "Ready" to be shown in the UI.
 *
 * A task is ready if its Lead Time Factor is greater than 0.
 * (i.e., we are within the window where timeRemaining <= 2 * leadTime).
 *
 * @param dueDate - The task's due date timestamp (ms), or undefined if no due date
 * @param leadTime - The lead time in milliseconds
 * @param currentTime - The current timestamp in milliseconds
 */
export function isTaskReady(
  dueDate: number | undefined,
  leadTime: number,
  currentTime: number,
): boolean {
  if (dueDate === undefined) {
    return true;
  }

  const timeRemaining = dueDate - currentTime;
  return timeRemaining <= 2 * leadTime;
}
