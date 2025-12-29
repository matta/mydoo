import type {ComputedTask, TaskID} from '../types';
import type {RootState} from './index';

/**
 * Selector for the task entities map.
 */
export function selectTaskEntities(
  state: RootState,
): Record<TaskID, ComputedTask> {
  return state.tasks.entities;
}

/**
 * Selector for the list of Todo IDs.
 */
export function selectTodoListIds(state: RootState): TaskID[] {
  return state.tasks.todoListIds;
}

/**
 * Selector for the last synchronized document.
 */
export function selectLastDoc(state: RootState) {
  return state.tasks.lastDoc;
}

/**
 * Selector for the prioritized "Do" list.
 * NOTE: This currently returns a new array on every call and requires memoization.
 * It is named for diagnostic purposes.
 */
export function selectTodoList(state: RootState): ComputedTask[] {
  const entities = selectTaskEntities(state);
  const todoListIds = selectTodoListIds(state);
  return todoListIds
    .map(id => entities[id])
    .filter((task): task is ComputedTask => !!task);
}

/**
 * Factory for a selector to retrieve a specific task by ID.
 * Returns a named function for better diagnostics in Redux DevTools/Warnings.
 */
export function selectTaskById(id: TaskID | undefined) {
  return function selectTask(state: RootState): ComputedTask | undefined {
    if (!id) return undefined;
    return state.tasks.entities[id];
  };
}
