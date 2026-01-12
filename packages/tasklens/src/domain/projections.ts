import type { ComputedTask, TaskFields, TaskID } from "../types/ui";
import { getCurrentTimestamp } from "../utils/time";
import { CREDITS_HALF_LIFE_MILLIS } from "./constants";
import { isTaskReady } from "./readiness";

/**
 * Projects a raw PersistedTask-like object into a ComputedTask for UI consumption.
 *
 * This performs a lightweight computation of visibility/status flags
 * to satisfy the UI contract without running the full priority algorithm.
 *
 * It uses generics to accept any type that satisfies TaskFields, allowing
 * it to bridge between strict UI types and loose persistence types.
 */
export function toComputedTask<T extends TaskFields>(task: T): ComputedTask {
  const currentTime = getCurrentTimestamp();
  const isPending = task.status === "Pending";
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
    effectiveDueDate: task.effectiveDueDate,
    effectiveLeadTime: task.effectiveLeadTime,
    effectiveScheduleSource: task.effectiveScheduleSource,
  } as ComputedTask;
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
export function getDescendantCountFromEntities<T extends TaskFields>(
  entities: Record<string, T>,
  taskId: TaskID,
): number {
  const task = entities[taskId as string];
  if (!task) return 0;

  let count = task.childTaskIds.length;
  for (const childId of task.childTaskIds) {
    count += getDescendantCountFromEntities(entities, childId);
  }
  return count;
}
