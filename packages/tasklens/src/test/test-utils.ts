/**
 * Test utilities for TaskLens.
 *
 * This module provides helper functions for creating test fixtures and
 * initializing test state.
 */

import type { TunnelState } from "../types/persistence";
import type { ComputedTask, TaskID as UITaskID } from "../types/ui";

/**
 * Creates a strict mock that throws if unexpected properties are accessed.
 *
 * This utility enforces that tests only use the methods they've explicitly mocked,
 * preventing tests from accidentally depending on implementation details.
 *
 * @param name - A descriptive name for the mock (used in error messages)
 * @param implementations - Partial implementation of the target type
 * @returns A proxy that behaves like T but throws on unexpected property access
 *
 * @example
 * ```typescript
 * const mockHandle = strictMock<DocHandle<TunnelState>>('DocHandle', {
 *   doc: () => myDoc,
 *   change: (fn) => fn(myDoc),
 * });
 * ```
 */
export function strictMock<T extends object>(
  name: string,
  implementations: Partial<T>,
): T {
  return new Proxy(implementations, {
    get: (target, prop) => {
      if (prop in target) {
        // biome-ignore lint/suspicious/noExplicitAny: Proxy requires dynamic access
        return (target as any)[prop];
      }
      // This enforces that tests only access what was explicitly mocked
      throw new Error(
        `[StrictMock: ${name}] Accessed unexpected property: '${String(prop)}'. \n` +
          `Only these were mocked: [${Object.keys(implementations).join(", ")}]`,
      );
    },
  }) as T;
}

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
    id: "test-task" as UITaskID,
    title: "Test Task",
    status: "Pending",
    importance: 0.5,
    childTaskIds: [],
    creditIncrement: 1,
    credits: 0,
    creditsTimestamp: 0,
    desiredCredits: 1.0,
    priorityTimestamp: 0,
    effectiveCredits: 0,
    schedule: { type: "Once", leadTime: 0 },
    isAcknowledged: false,
    isSequential: false,
    notes: "",
    isContainer: false,
    isPending: true,
    isReady: true,
    ...overrides,
  };

  // Automerge doesn't like undefined values being assigned.
  // We remove parentId if it's undefined to prevent RangeErrors when assigning to a doc.
  if (task.parentId === undefined) {
    const taskWithoutParent = task as { parentId?: UITaskID };
    delete taskWithoutParent.parentId;
  }

  return task;
}
