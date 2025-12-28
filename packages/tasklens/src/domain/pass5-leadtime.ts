import type {EnrichedTask} from '../../src/types';
import {calculateLeadTimeFactor} from './readiness';

/**
 * Pass 5: Lead Time Ramp
 * Calculates urgency based on deadlines.
 * Updates the `leadTimeFactor` property of each task.
 *
 * @param tasks All tasks in the document (Mutable EnrichedTasks).
 * @param currentTime The current timestamp in milliseconds.
 */
export function pass5LeadTimeRamp(
  tasks: EnrichedTask[],
  currentTime: number,
): void {
  tasks.forEach(task => {
    task.leadTimeFactor = calculateLeadTimeFactor(task.schedule, currentTime);
  });
}
