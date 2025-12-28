/**
 * TunnelStore: A wrapper around an Automerge document for algorithm testing.
 *
 * This class provides a higher-level API for managing the task state, primarily
 * used in unit tests for the prioritization algorithm. For UI integration,
 * use the `useTunnel` hook from `react.ts` instead.
 *
 * Key concepts:
 * - **Automerge.from(state)**: Creates a new Automerge document from an initial
 *   plain object. The document is a proxy that tracks all mutations.
 * - **Automerge.change(doc, callback)**: Applies mutations to the document
 *   within a tracked "change". This is how collaborative edits are captured
 *   and can be synced to other clients.
 * - **Automerge.save/load**: Serializes the document to/from a binary format
 *   for persistence.
 *
 * The store delegates actual mutations to the pure functions in `ops.ts`.
 */
import * as Automerge from '@automerge/automerge';

import {getPrioritizedTasks} from '../domain/priority';
import {
  ANYWHERE_PLACE_ID,
  type ComputedTask,
  type Context,
  type PersistedTask,
  type TaskID,
  type TunnelState,
  type ViewFilter,
} from '../types';
import * as TunnelOps from './ops';

/**
 * A store that wraps an Automerge document containing task state.
 *
 * This class is designed for use in unit tests and algorithm development.
 * For React applications, use the `useTunnel` hook instead, which provides
 * reactive updates when the document changes.
 *
 * @example
 * ```typescript
 * // Create a new store with empty state
 * const store = new TunnelStore();
 *
 * // Create a task
 * const task = store.createTask({ title: "Buy groceries" });
 *
 * // Update the task
 * store.updateTask(task.id, { importance: 0.8 });
 *
 * // Save to binary format
 * const bytes = store.save();
 * ```
 */
export class TunnelStore {
  /**
   * The underlying Automerge document.
   * This is a proxy object that tracks mutations for synchronization.
   */
  public doc: Automerge.Doc<TunnelState>;

  /**
   * Creates a new TunnelStore.
   *
   * @param initialState - Optional initial state. If not provided, creates
   *                       an empty state with no tasks or places.
   * @throws Error if the initial state contains the reserved "Anywhere" place ID.
   */
  constructor(initialState?: TunnelState) {
    if (initialState) {
      if (ANYWHERE_PLACE_ID in initialState.places) {
        throw new Error(
          `'${ANYWHERE_PLACE_ID}' is a reserved Place ID and cannot be defined.`,
        );
      }
      this.doc = Automerge.from(initialState);
    } else {
      this.doc = Automerge.from({
        tasks: {},
        places: {},
        rootTaskIds: [],
      });
    }
  }

  /**
   * Returns the current state as a read-only object.
   * Alias for `doc` for convenience.
   */
  get state(): TunnelState {
    return this.doc;
  }

  // --- CRUD Operations ---

  /**
   * Retrieves a task by ID.
   *
   * @param id - The ID of the task to retrieve.
   * @returns The Task object, or undefined if not found.
   */
  getTask(id: TaskID): PersistedTask | undefined {
    return TunnelOps.getTask(this.doc, id);
  }

  /**
   * Retrieves the immediate children of a task.
   *
   * @param parentId - The ID of the parent task, or undefined to get root tasks.
   * @returns An array of child Task objects in display order.
   */
  getChildren(parentId: TaskID | undefined): PersistedTask[] {
    return TunnelOps.getChildren(this.doc, parentId);
  }

  /**
   * Retrieves all ancestors of a task, from root to immediate parent.
   *
   * @param id - The ID of the task whose ancestors to retrieve.
   * @returns An array of ancestor Tasks, with the root first and immediate parent last.
   *
   * @example
   * // For a task hierarchy: A -> B -> C
   * store.getAncestors("C") // returns [A, B]
   */
  getAncestors(id: TaskID): PersistedTask[] {
    const ancestors: PersistedTask[] = [];
    let currentTask = this.getTask(id);
    while (currentTask?.parentId !== undefined) {
      const parent = this.getTask(currentTask.parentId);
      if (parent) {
        ancestors.unshift(parent);
        currentTask = parent;
      } else {
        break;
      }
    }
    return ancestors;
  }

  /**
   * Creates a new task and adds it to the state.
   *
   * @param props - Partial task properties. Omitted properties use defaults.
   * @returns The newly created Task object.
   * @throws Error if task creation fails.
   *
   * @example
   * const task = store.createTask({ title: "New task", parentId: "1" });
   */
  createTask(props: Partial<PersistedTask>): PersistedTask {
    let newTask: PersistedTask | undefined;
    this.doc = Automerge.change(this.doc, 'Create task', doc => {
      newTask = TunnelOps.createTask(doc, props);
    });
    if (!newTask) throw new Error('Failed to create task');

    const task = TunnelOps.getTask(this.doc, newTask.id);
    if (!task) throw new Error('Retrieved task is undefined');
    return task;
  }

  /**
   * Updates an existing task with new property values.
   *
   * @param id - The ID of the task to update.
   * @param props - Partial task properties to update.
   * @returns The updated Task object.
   * @throws Error if the task does not exist.
   */
  updateTask(id: TaskID, props: Partial<PersistedTask>): PersistedTask {
    this.doc = Automerge.change(this.doc, `Update task ${id}`, doc => {
      TunnelOps.updateTask(doc, id, props);
    });
    const task = TunnelOps.getTask(this.doc, id);
    if (!task) throw new Error('Retrieved task is undefined');
    return task;
  }

  /**
   * Marks a task as completed.
   *
   * @param id - The ID of the task to complete.
   */
  completeTask(id: TaskID): void {
    this.doc = Automerge.change(this.doc, `Complete task ${id}`, doc => {
      TunnelOps.completeTask(doc, id);
    });
  }

  /**
   * Deletes a task and all its descendants (cascade).
   *
   * @param id - The ID of the task to delete.
   * @returns The number of tasks deleted (task + descendants).
   */
  deleteTask(id: TaskID): number {
    let deletedCount = 0;
    this.doc = Automerge.change(this.doc, `Delete task ${id}`, doc => {
      deletedCount = TunnelOps.deleteTask(doc, id);
    });
    return deletedCount;
  }

  // --- Algorithm Operations ---

  /**
   * Returns a sorted list of visible tasks for display.
   *
   * Tasks are filtered to only those with `visibility: true` and a positive
   * priority score, then sorted by priority (highest first).
   *
   * @param _context - Runtime context (currently unused, reserved for future).
   * @returns An array of visible Tasks sorted by priority.
   */
  getTodoList(_context?: Context): ComputedTask[] {
    // getPrioritizedTasks inherently returns the Todo List (Sorted + Filtered for Status/Visibility)
    // Pass default filter if none provided
    return getPrioritizedTasks(this.doc, {});
  }

  /**
   * For Testing/Debugging: Get ALL calculated tasks including invalid/hidden ones.
   */
  dumpCalculatedState(viewFilter: ViewFilter = {}): ComputedTask[] {
    return getPrioritizedTasks(this.doc, viewFilter, {
      includeHidden: true,
      includeDone: true,
    });
  }

  // --- Persistence ---

  /**
   * Serializes the document to a binary format for storage.
   *
   * The returned Uint8Array can be persisted to disk, a database, or
   * transmitted over a network. Use `TunnelStore.load()` to restore.
   *
   * @returns A Uint8Array containing the serialized document.
   */
  save(): Uint8Array {
    return Automerge.save(this.doc);
  }

  /**
   * Restores a TunnelStore from a previously saved binary format.
   *
   * @param data - The Uint8Array from a previous `save()` call.
   * @returns A new TunnelStore instance with the restored state.
   *
   * @example
   * const bytes = existingStore.save();
   * const restoredStore = TunnelStore.load(bytes);
   */
  static load(data: Uint8Array): TunnelStore {
    const doc = Automerge.load<TunnelState>(data);
    const store = new TunnelStore();
    store.doc = doc;
    return store;
  }
}
