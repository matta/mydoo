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
 * @returns The newly created Task object.
 *
 * @throws Error if `parentId` refers to a non-existent task.
 * @throws Error if the parent task is already at maximum depth (20 levels).
 * @throws Error if `creditIncrement`, `importance`, or `desiredCredits` are invalid.
 *
 * @example
 * // Create a root task
 * createTask(state, { title: "Buy groceries" });
 *
 * // Create a child task
 * createTask(state, { title: "Buy milk", parentId: "1" });
 */
export function createTask(state: TunnelState, props: Partial<Task>): Task {
  // Use UUID for CRDT compatibility - sequential counters cause conflicts
  // when multiple replicas create tasks simultaneously.
  const newTaskId = crypto.randomUUID() as TaskID;

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
    creditIncrement: props.creditIncrement ?? 1.0,
    credits: props.credits ?? 0.0,
    desiredCredits: props.desiredCredits ?? 0.0,
    creditsTimestamp: props.creditsTimestamp ?? getCurrentTimestamp(),
    priorityTimestamp: props.priorityTimestamp ?? getCurrentTimestamp(),
    schedule: props.schedule ?? defaultSchedule,
    isSequential: props.isSequential ?? false,
    childTaskIds: [],
    // Remediation: Init as unacknowledged
    isAcknowledged: false,
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
  if (newTask.parentId) {
    const parent = state.tasks[newTask.parentId];
    if (parent) {
      // Validation: Depth limit
      validateDepth(state, newTask.parentId);

      parent.childTaskIds.unshift(newTaskId); // Add to top by default
    } else {
      throw new Error(`Parent task with ID ${newTask.parentId} not found.`);
    }
  } else {
    state.rootTaskIds.unshift(newTaskId);
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
 *
 * @example
 * // Mark task as important
 * updateTask(state, "1", { importance: 1.0 });
 *
 * // Move task to a new parent
 * updateTask(state, "1", { parentId: "2" });
 */
export function updateTask(
  state: TunnelState,
  id: TaskID,
  props: Partial<Task>,
): Task {
  const task = state.tasks[id];
  if (!task) {
    throw new Error(`Task with ID ${id} not found.`);
  }

  // Handle parentId change if it exists in props and is different
  if (props.parentId !== undefined && props.parentId !== task.parentId) {
    moveTask(state, id, props.parentId, undefined); // Move to top of new parent
    // Remove parentId from props to avoid double-setting it in the loop below
    // (moveTask handles it)
    // However, we still need to set other props.
    // Reflect.set will overwrite what moveTask did if we are not careful.
    // But moveTask sets task.parentId. props.parentId matches that. So it's fine.
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

  // Explicit property updates - fully type-safe, no escapes needed.
  // For required properties, 'in' check is sufficient (TypeScript knows value is defined).
  // For optional properties (parentId, placeId), we use 'in' to detect explicit undefined.
  if ('title' in props) task.title = props.title;
  if ('status' in props) task.status = props.status;
  if ('importance' in props) task.importance = props.importance;
  if ('creditIncrement' in props) task.creditIncrement = props.creditIncrement;
  if ('credits' in props) task.credits = props.credits;
  if ('desiredCredits' in props) task.desiredCredits = props.desiredCredits;
  if ('creditsTimestamp' in props)
    task.creditsTimestamp = props.creditsTimestamp;
  if ('priorityTimestamp' in props)
    task.priorityTimestamp = props.priorityTimestamp;
  if ('schedule' in props) task.schedule = props.schedule;
  if ('isSequential' in props) task.isSequential = props.isSequential;

  // Optional properties that can be deleted (Automerge requires delete, not undefined)
  if ('parentId' in props) {
    if (props.parentId === undefined) delete task.parentId;
    else task.parentId = props.parentId;
  }
  if ('placeId' in props) {
    if (props.placeId === undefined) delete task.placeId;
    else task.placeId = props.placeId;
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
 *
 * @example
 * // Move task to root (no parent), at the top of the list
 * moveTask(state, "5", undefined, undefined);
 *
 * // Move task under parent "2", after sibling "3"
 * moveTask(state, "5", "2", "3");
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
      // Sibling not found, append to end (or beginning? usually end implies safe default if target missing)
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
 *
 * @remarks
 * This is a simplified implementation. A full implementation would also:
 * - Award credits to the task and its ancestors.
 * - Handle recurring tasks by resetting status and advancing the due date.
 * - Update timestamps for priority recalculation.
 */
export function completeTask(state: TunnelState, id: TaskID): void {
  const task = state.tasks[id];
  if (!task) return;

  task.status = TaskStatus.Done;
}

/**
 * Deletes a task from the state.
 *
 * This function removes the task from its parent's child list (or the root list),
 * then removes it from the tasks map. Children of the deleted task are NOT
 * automatically deleted, which may result in orphaned tasks.
 *
 * @param state - The application state to mutate.
 * @param id - The ID of the task to delete.
 *
 * @remarks
 * If the task has children, a warning is logged to the console. The children
 * remain in the state but are no longer reachable from the tree structure.
 * A future improvement would recursively delete all descendants.
 *
 * The `delete` operator is used because `state.tasks` is a plain JavaScript
 * object (Record), which is required for Automerge compatibility. Automerge
 * cannot track changes to Map or Set types.
 */
export function deleteTask(state: TunnelState, id: TaskID): void {
  const task = state.tasks[id];
  if (!task) return;

  // Remove from parent's child list or root list
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

  // Remove task from state.
  //
  // Becuase state.tasks is an Automerge proxy, we use Reflect.deleteProperty to
  // remove the task. The usual way of doing this is to use delete
  // state.tasks[id]; or set it to undefined, but the former issues a lint
  // warning in our code base, and the latter is semantically wrong for
  // automerge proxies (it ends up setting the property to null but doesn't
  // delete it from the map).ß
  Reflect.deleteProperty(state.tasks, id);

  // TODO: Recursively delete children to avoid orphaned tasks
  if (task.childTaskIds.length > 0) {
    console.warn(
      `deleteTask: Task ${id} has ${String(
        task.childTaskIds.length,
      )} children that are now orphaned`,
    );
  }
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
 *
 * @example
 * const tree = getTaskTree(state);
 * // tree[0].title === "Work"
 * // tree[0].children[0].title === "Project A"
 */
export function getTaskTree(state: TunnelState): TunnelNode[] {
  const buildNode = (taskId: TaskID): TunnelNode | undefined => {
    const task = state.tasks[taskId];
    if (!task) return undefined;

    const children = task.childTaskIds
      .map(buildNode)
      .filter((n): n is TunnelNode => !!n);

    return {
      ...task,
      children,
    };
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
