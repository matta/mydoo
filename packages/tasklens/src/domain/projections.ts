import type {ComputedTask, PersistedTask} from '../types';
import {getCurrentTimestamp} from '../utils/time';
import {CREDITS_HALF_LIFE_MILLIS} from './constants';
import {isTaskReady} from './readiness';

/**
 * Projects a raw PersistedTask into a ComputedTask for UI consumption.
 *
 * This performs a lightweight computation of visibility/status flags
 * to satisfy the UI contract without running the full priority algorithm.
 *
 * logic matching `pass5LeadTimeRamp` and `pass1Visibility` (simplified).
 */
export function toComputedTask(task: PersistedTask): ComputedTask {
  const currentTime = getCurrentTimestamp();
  const isPending = task.status === 'Pending';
  const isContainer = task.childTaskIds.length > 0;

  const isReady = isPending && isTaskReady(task.schedule, currentTime);

  // Compute decayed credits
  const timeDelta = currentTime - task.creditsTimestamp;
  const effectiveCredits =
    task.credits * 0.5 ** (timeDelta / CREDITS_HALF_LIFE_MILLIS);

  return {
    ...task,
    effectiveCredits,
    isContainer,
    isPending,
    isReady,
  };
}
