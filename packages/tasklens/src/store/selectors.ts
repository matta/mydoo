import {createSelector} from '@reduxjs/toolkit';
import {calculateBalanceData} from '../domain/balance';
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
 * Redux selector for the prioritized "Do" list.
 *
 * @param state - The global Redux state.
 * @returns A referentially stable array of ComputedTask objects representing the current "Do" list.
 *
 * Computed by resolving the list of task IDs in the `todoListIds` slice to their
 * corresponding entities. Memoized to prevent unnecessary re-renders.
 */
export const selectTodoList = createSelector(
  // Input selectors: when these change, the selector will re-run the projection
  // function below.
  [selectTaskEntities, selectTodoListIds],
  projectTodoList,
);

/**
 * Pure projection function to resolve IDs to entity tasks.
 */
function projectTodoList(
  entities: Record<TaskID, ComputedTask>,
  todoListIds: TaskID[],
): ComputedTask[] {
  const todoList: ComputedTask[] = [];

  for (const id of todoListIds) {
    const task = entities[id];
    if (task) {
      todoList.push(task);
    }
  }

  return todoList;
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

/**
 * Selector for Balance View data.
 *
 * Computes the balance allocation for all root goals (excluding Inbox).
 * Returns targetPercent, actualPercent, and isStarving for each goal.
 */
export const selectBalanceData = createSelector(
  [selectTaskEntities],
  entities => calculateBalanceData(Object.values(entities)),
);
