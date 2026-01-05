import type {ComputedTask, PersistedTask, TaskID} from '../types';
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

/**
 * Calculates the total number of descendants for a given task from an entities record.
 *
 * This is a pure function that works with a normalized entities map, suitable for
 * use in React components that consume Redux state directly.
 *
 * @param entities - The normalized map of TaskID to task objects.
 * @param taskId - The ID of the task to count descendants for.
 * @returns The total number of sub-tasks (children, grandchildren, etc.).
 */
export function getDescendantCountFromEntities(
  entities: Record<TaskID, ComputedTask | PersistedTask>,
  taskId: TaskID,
): number {
  const task = entities[taskId];
  if (!task) return 0;

  let count = task.childTaskIds.length;
  for (const childId of task.childTaskIds) {
    count += getDescendantCountFromEntities(entities, childId);
  }
  return count;
}
