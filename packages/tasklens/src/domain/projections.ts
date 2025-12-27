import type {ComputedTask, PersistedTask} from '../types';

/**
 * Projects a raw PersistedTask into a ComputedTask for UI consumption.
 *
 * This performs a lightweight computation of visibility/status flags
 * to satisfy the UI contract without running the full priority algorithm.
 *
 * logic matching `pass5LeadTimeRamp` and `pass1Visibility` (simplified).
 */
export function toComputedTask(task: PersistedTask): ComputedTask {
  const isPending = task.status === 'Pending';
  const isContainer = task.childTaskIds.length > 0;

  // Approximate isReady logic:
  // Must match pass5LeadTimeRamp logic:
  // - If no due date -> Neutral (Ready)
  // - If due date present:
  //    - Ready if timeRemaining <= 2 * leadTime
  //      (i.e. we are inside the ramp-up window)
  let isReady = false;

  if (isPending) {
    if (task.schedule.dueDate === undefined) {
      isReady = true;
    } else {
      const now = Date.now();
      const leadTime = task.schedule.leadTime;
      // Start Ramp = DueDate - 2 * LeadTime
      // We are ready if Now >= Start Ramp
      const startRamp = task.schedule.dueDate - 2 * leadTime;
      isReady = now >= startRamp;
    }
  }

  return {
    ...task,
    isContainer,
    isPending,
    isReady,
  };
}
