/**
 * Test utilities for TaskLens.
 *
 * This module provides helper functions for creating test fixtures and
 * initializing test state.
 */

import type {ComputedTask, TaskID, TunnelState} from './types';

/**
 * Creates an empty TunnelState document.
 *
 * This is useful for initializing test repositories with a valid document
 * structure that passes schema validation.
 *
 * @returns A minimal valid TunnelState with no tasks or places.
 *
 * @example
 * ```typescript
 * const repo = new Repo({network: []});
 * const handle = repo.create(createEmptyTunnelState());
 * ```
 */
export function createEmptyTunnelState(): TunnelState {
  return {
    tasks: {},
    rootTaskIds: [],
    places: {},
  };
}

/**
 * Creates a mock ComputedTask with sensible defaults for tests.
 *
 * @param overrides - Optional object to override specific task properties.
 *                    Merged with the default mock values.
 * @returns A complete ComputedTask object.
 */
export function createMockTask(
  overrides: Partial<ComputedTask> = {},
): ComputedTask {
  const task: ComputedTask = {
    id: 'test-task' as TaskID,
    title: 'Test Task',
    status: 'Pending',
    importance: 0.5,
    childTaskIds: [],
    creditIncrement: 1,
    credits: 0,
    creditsTimestamp: 0,
    desiredCredits: 1.0,
    priorityTimestamp: 0,
    effectiveCredits: 0,
    schedule: {type: 'Once', leadTime: 0},
    isAcknowledged: false,
    isSequential: false,
    notes: '',
    isContainer: false,
    isPending: true,
    isReady: true,
    ...overrides,
  };

  // Automerge doesn't like undefined values being assigned.
  // We remove parentId if it's undefined to prevent RangeErrors when assigning to a doc.
  if (task.parentId === undefined) {
    const taskWithoutParent = task as {parentId?: TaskID};
    delete taskWithoutParent.parentId;
  }

  return task;
}
