/**
 * Validation invariants for the Tasklens domain.
 * These are pure functions that throw errors if invariants are violated.
 */
import type {TaskID, TunnelState} from '../types';

/**
 * Validates that adding a task to a parent does not exceed the maximum depth limit.
 * @param state The current state
 * @param parentId The ID of the parent task
 * @param limit The maximum allowed depth (default 20)
 * @param errorPrefix Custom error message prefix (default "Cannot create task: parent")
 * @throws Error if depth limit is exceeded
 */
export function validateDepth(
  state: TunnelState,
  parentId: TaskID,
  limit = 20,
  errorPrefix = 'Cannot create task: parent',
): void {
  let parentDepth = 0;
  let p: TaskID | undefined = parentId;
  while (p) {
    parentDepth++;
    p = state.tasks[p]?.parentId ?? undefined;
    if (parentDepth > limit) {
      throw new Error(
        `${errorPrefix} already at maximum hierarchy depth (${String(limit)}).`,
      );
    }
  }
}

/**
 * Validates that moving a task to a new parent does not create a cycle.
 * @param state The current state
 * @param taskId The ID of the task being moved
 * @param newParentId The ID of the new parent
 * @throws Error if a cycle is detected
 */
export function validateNoCycle(
  state: TunnelState,
  taskId: TaskID,
  newParentId: TaskID,
): void {
  let currentId: TaskID | undefined = newParentId;
  while (currentId) {
    if (currentId === taskId) {
      throw new Error(
        `Cannot move task ${taskId} into its own descendant ${newParentId}.`,
      );
    }
    currentId = state.tasks[currentId]?.parentId ?? undefined;
  }
}
