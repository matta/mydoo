/**
 * Pure state mutation operations for the Tunnel data model.
 *
 * This module contains functions that directly mutate a `TunnelState` object.
 * These functions are designed to be called within Automerge change callbacks,
 * where mutations to the document proxy are tracked and synchronized.
 *
 * Key operations:
 * - **createTask**: Adds a new task to the state, validating parent existence
 *   and hierarchy depth limits (max 20 levels).
 * - **updateTask**: Modifies properties of an existing task.
 * - **moveTask**: Relocates a task to a new parent or position, with cycle
 *   detection to prevent a task from becoming its own ancestor.
 * - **deleteTask**: Removes a task from the state. Currently does not delete
 *   children recursively (documented as a TODO).
 *
 * The task tree is maintained via two structures:
 * - `state.rootTaskIds`: Ordered list of top-level task IDs with no parent.
 * - `task.childTaskIds`: Ordered list of child task IDs for each task.
 *
 * All functions assume they are operating on an Automerge proxy (or a plain
 * object in tests). The `state.tasks` Record uses plain objects because
 * Automerge cannot proxy JavaScript Map or Set types.
 */
import {validateDepth, validateNoCycle} from '../domain/invariants';
import {
  ANYWHERE_PLACE_ID,
  type CreateTaskOptions,
  type Schedule,
  type Task,
  type TaskID,
  TaskStatus,
  type TunnelNode,
  type TunnelState,
} from '../types';
import {daysToMilliseconds, getCurrentTimestamp} from '../utils/time';

// --- Mutators ---

/**
 * Creates a new task and adds it to the state.
 *
 * This function generates a unique ID, initializes all task properties with
 * defaults (or values from `props`), validates the input, and inserts the
 * task into the appropriate location in the tree structure.
 *
 * @param state - The application state to mutate. Must be an Automerge proxy
 *                or a plain object for testing.
 * @param props - Partial task properties. Any omitted properties will use defaults.
 *                Common properties to set: `title`, `parentId`, `importance`.
 * @param options - Optional positioning configuration.
 * @returns The newly created Task object.
 *
 * @throws Error if `parentId` refers to a non-existent task.
 * @throws Error if the parent task is already at maximum depth (20 levels).
 * @throws Error if `creditIncrement`, `importance`, or `desiredCredits` are invalid.
 */
export function createTask(
  state: TunnelState,
  props: Partial<Task>,
  options: CreateTaskOptions = {position: 'end'},
): Task {
  // Use UUID for CRDT compatibility - sequential counters cause conflicts
  // when multiple replicas create tasks simultaneously.
  // Caller may provide an ID for testing purposes.
  const newTaskId = props.id ?? (crypto.randomUUID() as TaskID);
  const defaultSchedule: Schedule = {
    type: 'Once',
    dueDate: undefined,
    leadTime: daysToMilliseconds(7),
  };
  const newTask: Task = {
    id: newTaskId,
    title: props.title ?? 'New Task',
    parentId: props.parentId ?? undefined,
    // Inherit placeId from parent, or default to ANYWHERE_PLACE_ID for root tasks
    placeId:
      props.placeId ??
      (props.parentId
        ? state.tasks[props.parentId]?.placeId
        : ANYWHERE_PLACE_ID) ??
      ANYWHERE_PLACE_ID,
    status: props.status ?? TaskStatus.Pending,
    importance: props.importance ?? 1.0,
    creditIncrement: props.creditIncrement ?? 0.5,
    credits: props.credits ?? 0.0,
    desiredCredits: props.desiredCredits ?? 0.0,
    creditsTimestamp: props.creditsTimestamp ?? getCurrentTimestamp(),
    priorityTimestamp: props.priorityTimestamp ?? getCurrentTimestamp(),
    schedule: props.schedule ?? defaultSchedule,
    isSequential: props.isSequential ?? false,
    childTaskIds: [],
    // Remediation: Init as unacknowledged
    isAcknowledged: false,
    notes: props.notes ?? '',
  };

  // Automerge doesn't support 'undefined' values, so we must remove them
  if (newTask.parentId === undefined) delete newTask.parentId;
  // placeId now always has a value from inheritance logic, but check anyway
  if (newTask.placeId === undefined) delete newTask.placeId;
  if (newTask.schedule.dueDate === undefined) delete newTask.schedule.dueDate;

  // Validations for numbers
  if (newTask.creditIncrement < 0)
    throw new Error('CreditIncrement cannot be negative.');
  if (newTask.importance < 0 || newTask.importance > 1)
    throw new Error('Importance must be between 0.0 and 1.0.');
  if (newTask.desiredCredits < 0)
    throw new Error('DesiredCredits cannot be negative.');

  state.tasks[newTaskId] = newTask;

  // Add to parent's children list or root list
  let targetList: TaskID[];
  if (newTask.parentId) {
    const parent = state.tasks[newTask.parentId];
    if (parent) {
      // Validation: Depth limit
      validateDepth(state, newTask.parentId);
      targetList = parent.childTaskIds;
    } else {
      throw new Error(`Parent task with ID ${newTask.parentId} not found.`);
    }
  } else {
    targetList = state.rootTaskIds;
  }

  // Position the task
  if (options.position === 'start') {
    targetList.unshift(newTaskId);
  } else if (options.position === 'after' && options.afterTaskId) {
    const idx = targetList.indexOf(options.afterTaskId);
    if (idx !== -1) {
      targetList.splice(idx + 1, 0, newTaskId);
    } else {
      // Fallback to end if afterTaskId not found
      targetList.push(newTaskId);
    }
  } else {
    // Default: end
    targetList.push(newTaskId);
  }
  return newTask;
}

/**
 * Updates an existing task with new property values.
 *
 * If `props.parentId` is provided and differs from the current parent,
 * the task is automatically moved to the new parent (using `moveTask`).
 *
 * @param state - The application state to mutate.
 * @param id - The ID of the task to update.
 * @param props - Partial task properties to update. Properties not specified
 *                will retain their current values. Cannot update `id` or `childTaskIds`.
 * @returns The updated Task object.
 *
 * @throws Error if the task with the given ID does not exist.
 * @throws Error if any numeric property is outside its valid range.
 */
export function updateTask(
  state: TunnelState,
  id: TaskID,
  props: Partial<Task>,
): Task {
  const task = state.tasks[id];
  if (!task) throw new Error(`Task with ID ${id} not found.`);

  // Handle parentId change if it exists in props and is different
  if (props.parentId !== undefined && props.parentId !== task.parentId) {
    moveTask(state, id, props.parentId, undefined); // Move to top of new parent
    // Note: moveTask updates task.parentId in place.
  }

  // Validation for numeric props
  if (props.desiredCredits !== undefined && props.desiredCredits < 0) {
    throw new Error('DesiredCredits cannot be negative.');
  }
  if (props.creditIncrement !== undefined && props.creditIncrement < 0) {
    throw new Error('CreditIncrement cannot be negative.');
  }
  if (
    props.importance !== undefined &&
    (props.importance < 0 || props.importance > 1)
  ) {
    throw new Error('Importance must be between 0.0 and 1.0.');
  }

  // Idiomatic Automerge: Mutate the proxy directly
  // This should trigger the change detection.
  if (props.title !== undefined) task.title = props.title;
  if (props.status !== undefined) task.status = props.status;
  if (props.importance !== undefined) task.importance = props.importance;
  if (props.creditIncrement !== undefined)
    task.creditIncrement = props.creditIncrement;
  if (props.credits !== undefined) task.credits = props.credits;
  if (props.desiredCredits !== undefined)
    task.desiredCredits = props.desiredCredits;
  if (props.creditsTimestamp !== undefined)
    task.creditsTimestamp = props.creditsTimestamp;
  if (props.priorityTimestamp !== undefined)
    task.priorityTimestamp = props.priorityTimestamp;
  if (props.isSequential !== undefined) task.isSequential = props.isSequential;
  if (props.isAcknowledged !== undefined)
    task.isAcknowledged = props.isAcknowledged;
  if (props.notes !== undefined) task.notes = props.notes;

  // Handle nested objects carefully
  if (props.repeatConfig !== undefined) {
    task.repeatConfig = props.repeatConfig;
  } else if ('repeatConfig' in props) {
    delete task.repeatConfig;
  }

  if (props.schedule) {
    if (props.schedule.type) task.schedule.type = props.schedule.type;
    if (props.schedule.leadTime !== undefined)
      task.schedule.leadTime = props.schedule.leadTime;

    // Only delete dueDate if explicitly passed as undefined (use 'in' check)
    if ('dueDate' in props.schedule) {
      if (props.schedule.dueDate === undefined) {
        delete task.schedule.dueDate;
      } else {
        task.schedule.dueDate = props.schedule.dueDate;
      }
    }
  }

  // Handle optional properties that can be unset (Automerge requires delete, not undefined)
  if (props.parentId === undefined && 'parentId' in props) {
    delete task.parentId;
    // Note: If we just nullified parentId but didn't call moveTask, the task
    // might still be in the old parent's child list.
    // However, the earlier check `if (props.parentId !== undefined ...)` handles explicit moves.
    // If user passed explicit `parentId: undefined`, that block handles moving to root.
  }

  if (props.placeId === undefined && 'placeId' in props) {
    delete task.placeId;
  } else if (props.placeId !== undefined) {
    task.placeId = props.placeId;
  }
  return task;
}

/**
 * Moves a task to a new location in the tree hierarchy.
 *
 * This function:
 * 1. Validates that the move doesn't create a cycle (task becoming its own ancestor).
 * 2. Validates that the new parent isn't already at maximum depth.
 * 3. Removes the task from its current location (old parent's children or root list).
 * 4. Inserts the task into the new location at the specified position.
 * 5. Updates the task's `parentId` pointer.
 *
 * @param state - The application state to mutate.
 * @param id - The ID of the task to move.
 * @param newParentId - The ID of the new parent task, or undefined to make it a root task.
 * @param afterTaskId - The ID of a sibling task to insert after, or undefined to insert
 *                      at the beginning (top) of the list.
 *
 * @throws Error if the task with the given ID does not exist.
 * @throws Error if moving would create a cycle (task becoming ancestor of itself).
 * @throws Error if the new parent is at maximum hierarchy depth (20 levels).
 * @throws Error if the new parent task does not exist.
 */
export function moveTask(
  state: TunnelState,
  id: TaskID,
  newParentId: TaskID | undefined,
  afterTaskId: TaskID | undefined,
): void {
  const task = state.tasks[id];
  if (!task) throw new Error(`Task ${id} not found`);

  // Validation: Cycle detection
  if (newParentId) {
    validateNoCycle(state, id, newParentId);
  }

  // Validation: Depth limit
  const parentIdCheck = newParentId ?? undefined;
  if (parentIdCheck) {
    validateDepth(state, parentIdCheck, 20, 'Cannot move task: new parent');
  }

  const oldParentId = task.parentId;

  // 1. Remove from old location
  if (oldParentId) {
    const oldParent = state.tasks[oldParentId];
    // oldParent is guaranteed to exist if oldParentId is not undefined, based on state integrity.
    const idx = oldParent?.childTaskIds.indexOf(id);
    if (idx !== undefined && idx !== -1) oldParent?.childTaskIds.splice(idx, 1);
  } else {
    // Was root
    const idx = state.rootTaskIds.indexOf(id);
    if (idx !== -1) state.rootTaskIds.splice(idx, 1);
  }

  // 2. Add to new location
  let targetList: TaskID[];
  if (newParentId) {
    const newParent = state.tasks[newParentId];
    if (!newParent) throw new Error(`New parent ${newParentId} not found`);
    targetList = newParent.childTaskIds;
  } else {
    targetList = state.rootTaskIds;
  }

  if (afterTaskId) {
    const afterIdx = targetList.indexOf(afterTaskId);
    if (afterIdx === -1) {
      // Sibling not found, append to end (safe default)
      targetList.push(id);
    } else {
      targetList.splice(afterIdx + 1, 0, id);
    }
  } else {
    // Insert at beginning
    targetList.unshift(id);
  }

  // 3. Update task parent pointer
  if (newParentId === undefined) {
    delete task.parentId;
  } else {
    task.parentId = newParentId;
  }
}

/**
 * Marks a task as completed by setting its status to Done.
 *
 * @param state - The application state to mutate.
 * @param id - The ID of the task to complete.
 */
export function completeTask(state: TunnelState, id: TaskID): void {
  const task = state.tasks[id];
  if (!task) return;
  task.status = TaskStatus.Done;
}

/**
 * Deletes a task and all its descendants (cascade hard-delete).
 *
 * This function uses an iterative queue-based approach (not recursion)
 * to safely handle deeply nested hierarchies without stack overflow.
 *
 * Steps:
 * 1. Removes the task from its parent's childTaskIds (or rootTaskIds).
 * 2. Iteratively removes all descendants from the tasks map.
 *
 * @param state - The application state to mutate.
 * @param id - The ID of the task to delete.
 * @returns The count of deleted tasks (target + descendants).
 */
export function deleteTask(state: TunnelState, id: TaskID): number {
  const task = state.tasks[id];
  if (!task) return 0;

  // 1. Remove from parent's child list or root list FIRST
  if (task.parentId) {
    const parent = state.tasks[task.parentId];
    if (parent?.childTaskIds) {
      const idx = parent.childTaskIds.indexOf(id);
      if (idx > -1) parent.childTaskIds.splice(idx, 1);
    }
  } else {
    const idx = state.rootTaskIds.indexOf(id);
    if (idx > -1) state.rootTaskIds.splice(idx, 1);
  }

  // 2. Iteratively delete target and all descendants using a queue
  const queue: TaskID[] = [id];
  let deletedCount = 0;

  while (queue.length > 0) {
    const taskId = queue.shift();
    if (taskId === undefined) continue;
    const t = state.tasks[taskId];
    if (!t) continue;

    // Queue all children for deletion
    queue.push(...t.childTaskIds);

    // Delete this task
    Reflect.deleteProperty(state.tasks, taskId);
    deletedCount += 1;
  }
  return deletedCount;
}

// --- Selectors (read-only queries) ---

/**
 * Retrieves a task by its ID.
 *
 * @param state - The application state to read from.
 * @param id - The ID of the task to retrieve.
 * @returns The Task object, or undefined if not found.
 */
export function getTask(state: TunnelState, id: TaskID): Task | undefined {
  return state.tasks[id];
}

/**
 * Retrieves the immediate children of a task.
 *
 * @param state - The application state to read from.
 * @param parentId - The ID of the parent task, or undefined to get root-level tasks.
 * @returns An array of Task objects in their display order. Returns empty array
 *          if the parent has no children or doesn't exist.
 */
export function getChildren(
  state: TunnelState,
  parentId: TaskID | undefined,
): Task[] {
  const ids = parentId
    ? state.tasks[parentId]?.childTaskIds
    : state.rootTaskIds;
  if (!ids) return [];
  return ids.map(id => state.tasks[id]).filter((t): t is Task => !!t);
}

/**
 * Builds the complete task tree from the flat state.
 *
 * This function recursively constructs a tree structure where each TunnelNode
 * contains its resolved children. This is useful for rendering the task
 * hierarchy in a user interface.
 *
 * @param state - The application state to read from.
 * @returns An array of root-level TunnelNodes, each with nested children.
 *
 * @remarks
 * The function performs a depth-first traversal starting from `rootTaskIds`.
 * Tasks that don't exist in the state (e.g., corrupted references) are
 * filtered out to maintain a valid tree structure.
 */
export function getTaskTree(state: TunnelState): TunnelNode[] {
  const buildNode = (taskId: TaskID): TunnelNode | undefined => {
    const task = state.tasks[taskId];
    if (!task) return undefined;
    const children = task.childTaskIds
      .map(buildNode)
      .filter((n): n is TunnelNode => !!n);
    return {...task, children};
  };
  return state.rootTaskIds.map(buildNode).filter((n): n is TunnelNode => !!n);
}

/**
 * Calculates the total number of descendants for a given task.
 *
 * @param state - The application state to read from.
 * @param taskId - The ID of the task to start from.
 * @returns The total number of sub-tasks (children, grandchildren, etc.).
 */
export function getDescendantCount(state: TunnelState, taskId: TaskID): number {
  const task = state.tasks[taskId];
  if (!task) return 0;
  let count = task.childTaskIds.length;
  for (const childId of task.childTaskIds) {
    count += getDescendantCount(state, childId);
  }
  return count;
}
