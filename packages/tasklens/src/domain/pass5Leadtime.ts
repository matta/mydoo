import type {Task} from '../../src/types';

/**
 * Pass 5: Lead Time Ramp
 * Calculates urgency based on deadlines.
 * Updates the `leadTimeFactor` property of each task.
 *
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param currentTime The current timestamp in milliseconds.
 */
export function pass5LeadTimeRamp(tasks: Task[], currentTime: number): void {
  tasks.forEach(task => {
    // If no schedule or no dueDate, leadTimeFactor is neutral (1.0 or 0.0 depending on desired default for un-scheduled tasks)
    // The spec example shows 0.0 for "Too early (Hidden)", suggesting un-scheduled are hidden.
    // For now, let's assume if there's no dueDate, it's not lead-time-ramped, so factor is 1.0 (neutral) or 0.0 (hidden).
    // The spec "IsReady" condition: `CurrentTime >= DueDate - (2 * LeadTime)`.
    // If no dueDate, cannot be ready by this definition.

    if (task.schedule.dueDate === undefined) {
      task.leadTimeFactor = 1.0; // Neutral, not ramping if no due date
      return;
    }

    const dueDate = task.schedule.dueDate;
    const leadTimeMillis = task.schedule.leadTime;

    const timeRemaining = dueDate - currentTime;
    let rawFactor = 0.0; // Initialize rawFactor

    // Ramp Function
    if (timeRemaining > 2 * leadTimeMillis) {
      task.leadTimeFactor = 0.0; // Too early
    } else {
      rawFactor = (2 * leadTimeMillis - timeRemaining) / leadTimeMillis;
      task.leadTimeFactor = Math.max(0, Math.min(1, rawFactor));
    }
  });
}
