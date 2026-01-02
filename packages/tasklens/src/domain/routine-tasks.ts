import type {DocHandle} from '@automerge/automerge-repo';
import type {TunnelState} from '../types';

/**
 * Wake up "Routinely" tasks that are due for their next cycle.
 *
 * This function is triggered manually by the user (e.g. via "Refresh").
 * It checks all "Done" + "Routinely" tasks to see if their wake-up window has arrived.
 * If so, it resets them to "Pending" and updates their due date to the next interval.
 */
export function wakeUpRoutineTasks(handle: DocHandle<TunnelState>) {
  handle.change(doc => {
    const now = Date.now();

    for (const task of Object.values(doc.tasks)) {
      if (task.status === 'Done' && task.schedule?.type === 'Routinely') {
        const repeatConfig = task.repeatConfig;

        // Safety check: Routinely tasks must have a repeat config
        if (!repeatConfig) {
          continue;
        }

        const lastCompletedAt = task.lastCompletedAt ?? 0;

        // Calculate the next theoretical due date based on completion time + interval
        const intervalMs = getIntervalMs(
          repeatConfig.frequency,
          repeatConfig.interval,
        );
        const nextDueDate = lastCompletedAt + intervalMs;

        // Lead Time defines how early the task appears before it's due
        const wakeUpTime = nextDueDate - task.schedule.leadTime;

        if (now >= wakeUpTime) {
          // Wake up the task!
          task.status = 'Pending';
          task.isAcknowledged = false;

          // Update the schedule for the new cycle
          task.schedule.dueDate = nextDueDate;
        }
      }
    }
  });
}

/**
 * Helper: Convert frequency/interval to milliseconds.
 * Note: simplified logic for MVP (ignoring leap years/DST nuances for basic "days/weeks").
 */
function getIntervalMs(
  frequency: 'daily' | 'weekly' | 'monthly' | 'yearly',
  interval: number,
): number {
  const ONE_DAY_MS = 24 * 60 * 60 * 1000;

  switch (frequency) {
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
