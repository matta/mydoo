import type {ComputedTask, PersistedTask} from '../types';
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
  const isPending = task.status === 'Pending';
  const isContainer = task.childTaskIds.length > 0;

  const isReady = isPending && isTaskReady(task.schedule, Date.now());

  return {
    ...task,
    isContainer,
    isPending,
    isReady,
  };
}
