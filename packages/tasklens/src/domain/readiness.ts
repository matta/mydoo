import type { ScheduleFields } from "../types/ui";

/**
 * Calculates the Lead Time Factor for a task based on its schedule and the current time.
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
 */
export function calculateLeadTimeFactor(
  schedule: ScheduleFields,
  currentTime: number,
): number {
  if (schedule.dueDate === undefined) {
    return 1.0;
  }

  const dueDate = schedule.dueDate;
  const leadTimeMillis = schedule.leadTime;
  const timeRemaining = dueDate - currentTime;

  if (timeRemaining > 2 * leadTimeMillis) {
    return 0.0;
  }

  const rawFactor = (2 * leadTimeMillis - timeRemaining) / leadTimeMillis;
  return Math.max(0, Math.min(1, rawFactor));
}

/**
 * Determines if a task is "Ready" to be shown in the UI.
 *
 * A task is ready if its Lead Time Factor is greater than 0.
 * (i.e., we are within the window where timeRemaining <= 2 * leadTime).
 */
export function isTaskReady(
  schedule: ScheduleFields,
  currentTime: number,
): boolean {
  if (schedule.dueDate === undefined) {
    return true;
  }

  const dueDate = schedule.dueDate;
  const leadTimeMillis = schedule.leadTime;
  const timeRemaining = dueDate - currentTime;

  return timeRemaining <= 2 * leadTimeMillis;
}
