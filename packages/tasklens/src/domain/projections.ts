import {type Task, TaskStatus, type TunnelState} from '../types';

/**
 * Projects the raw state into a sorted, prioritized list of pending tasks.
 *
 * Logic:
 * 1. Filter for Pending tasks.
 * 2. Sort by Computed Priority (descending).
 *
 * @param state The full application state.
 * @returns Array of sorted Task objects.
 */
export function selectPriorityList(state: TunnelState): Task[] {
  return Object.values(state.tasks)
    .filter(t => t.status === TaskStatus.Pending)
    .sort(
      (a, b) => (b.priority ?? b.importance) - (a.priority ?? a.importance),
    );
}
