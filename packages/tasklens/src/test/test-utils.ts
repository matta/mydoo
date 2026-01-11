/**
 * Test utilities for TaskLens.
 *
 * This module provides helper functions for creating test fixtures and
 * initializing test state.
 */

import type { DocHandle } from "@automerge/automerge-repo";
import { Repo } from "@automerge/automerge-repo";

import { initializeTunnelState } from "../domain/initialization";
import { createTask, updateTask } from "../persistence/ops";
import { TunnelStateSchema } from "../persistence/schemas";
import { createTaskLensStore } from "../store/index";
import type {
  TaskCreateInput,
  TaskID,
  TunnelState,
} from "../types/persistence";

import type { ComputedTask } from "../types/ui";

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
 * Gets the number of tasks in a document (validates schema first).
 *
 * @throws Error if the document does not match the TunnelState schema.
 */
export function getTaskCount(doc: unknown): number {
  const result = TunnelStateSchema.safeParse(doc);

  if (!result.success) {
    throw new Error(
      `getTaskCount failed: Document does not match schema.\n${result.error.message}`,
    );
  }

  return Object.keys(result.data.tasks).length;
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
    id: "test-task" as TaskID,
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
    const taskWithoutParent = task as { parentId?: TaskID };
    delete taskWithoutParent.parentId;
  }

  return task;
}

/**
 * Creates a mock/detached document handle for testing.
 *
 * @param repo - The Automerge Repo instance.
 * @returns A DocHandle suitable for testing with full type visibility.
 */
export function createMockTaskLensDoc(repo: Repo) {
  const handle = repo.create<TunnelState>();
  handle.change(initializeTunnelState);
  return handle;
}

/**
 * Creates a complete test environment with a Repo, Document, and Redux Store.
 *
 * @param repo - Optional Repo instance. If not provided, a fresh in-memory repo is created.
 * @returns Object containing the repo, document handle, docUrl, and configured store.
 */
export function createTaskLensTestEnvironment(
  repo: Repo = new Repo({ network: [] }),
) {
  const handle = createMockTaskLensDoc(repo);
  const docUrl = handle.url;
  const store = createTaskLensStore(repo, docUrl);

  return {
    repo,
    handle,
    docUrl,
    store,
  };
}

/**
 * Seeds a task into an Automerge document using strict validation.
 *
 * This wrapper uses the actual application logic (`TunnelOps.createTask`)
 * to ensure that test data complies with all invariants (e.g. hierarchy depth,
 * valid field ranges).
 *
 * @param handle - The Automerge document handle.
 * @param props - Task properties. Required fields (like title) are defaulted if omitted.
 * @returns The ID of the created task.
 */
export function seedTask(
  handle: DocHandle<TunnelState>,
  props: Omit<Partial<TaskCreateInput>, "id"> & { id: string },
): TaskID {
  const id = props.id as TaskID;
  const { id: _idStr, ...rest } = props;
  const input: TaskCreateInput = {
    title: "Test Task",
    ...rest,
    id,
  };

  handle.change((doc) => {
    createTask(doc, input);
    if (input.isAcknowledged) {
      updateTask(doc, id, { isAcknowledged: true });
    }
  });

  return id;
}

export { mockCurrentTimestamp, resetCurrentTimestampMock } from "../utils/time";
