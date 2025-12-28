import {
  type CreateTaskOptions,
  type DocumentHandle,
  type Task,
  type TaskID,
  TaskStatus,
  useTunnel,
} from '@mydoo/tasklens';

import {useCallback} from 'react';

/**
 * Hook to manage user intentions for Tasks.
 *
 * This hook acts as a facade over the generic `useTunnel` operations,
 * providing domain-specific actions like "createTask" and "toggleTaskCompletion".
 */
export function useTaskIntents(docUrl: DocumentHandle) {
  const {doc, ops} = useTunnel(docUrl);

  const createTask = useCallback(
    (
      title: string,
      parentId?: TaskID,
      options?: CreateTaskOptions,
      props?: Partial<Task>,
    ): TaskID => {
      // Generate ID client-side so we can return it and use it for navigation/highlight
      const newTaskId = crypto.randomUUID() as TaskID;
      ops.add({id: newTaskId, title, parentId, ...props}, options);
      return newTaskId;
    },
    [ops],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      ops.delete(id);
    },
    [ops],
  );

  const updateTask = useCallback(
    (id: TaskID, updates: Partial<Task>) => {
      ops.update(id, updates);
    },
    [ops],
  );

  /**
   * Moves a task to a new parent and/or position.
   */
  const moveTask = useCallback(
    (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => {
      ops.move(id, newParentId, afterTaskId);
    },
    [ops],
  );

  /**
   * Demotes a task to become a child of its previous sibling.
   *
   * @param id - The ID of the task to indent.
   *
   * @remarks
   * This operation reparents the task to the previous sibling node, appending it
   * to that node's existing children.
   *
   * Constraints:
   * - Cannot indent the first child in a list (no previous sibling).
   * - The task becomes the last child of the new parent.
   */
  const indentTask = useCallback(
    (id: TaskID) => {
      if (!doc) return;

      const task = doc.tasks[id];
      if (!task) return;

      // Determine the list of siblings to find the predecessor
      const siblings = task.parentId
        ? doc.tasks[task.parentId]?.childTaskIds
        : doc.rootTaskIds;

      if (!siblings) return;

      const idx = siblings.indexOf(id);
      // Cannot indent if first child (idx 0) or not found (-1)
      if (idx <= 0) return;

      const prevSiblingId = siblings[idx - 1];
      if (!prevSiblingId) return;

      const prevSibling = doc.tasks[prevSiblingId];
      if (!prevSibling) return;

      // Move to become the last child of the previous sibling
      // We need the last child ID of the new parent to append.
      const newParentLastChildId =
        prevSibling.childTaskIds.length > 0
          ? prevSibling.childTaskIds[prevSibling.childTaskIds.length - 1]
          : undefined;

      ops.move(id, prevSiblingId, newParentLastChildId);
    },
    [doc, ops],
  );

  /**
   * Promotes a task to be a sibling of its current parent.
   *
   * @param id - The ID of the task to outdent.
   *
   * @remarks
   * This operation moves the task to the level of its parent, placing it immediately
   * after the parent in the sibling order.
   *
   * Constraints:
   * - Cannot outdent a root task (has no parent).
   * - Finds the parent and reparents the task to the parent's parent (or root).
   */
  const outdentTask = useCallback(
    (id: TaskID) => {
      if (!doc) return;

      const task = doc.tasks[id];
      if (!task) return;

      const parentId = task.parentId;
      if (!parentId) return; // Already at root, cannot outdent

      const parent = doc.tasks[parentId];
      if (!parent) return;

      // Move to be a sibling of the parent, immediately following it.
      ops.move(id, parent.parentId, parentId);
    },
    [doc, ops],
  );

  /**
   * Toggles the completion status of a task between 'Done' and 'Pending'.
   * @param id - The ID of the task to toggle.
   */
  const toggleTask = useCallback(
    (id: TaskID) => {
      if (!doc) return;
      const task = doc.tasks[id];
      if (!task) return;

      const newStatus =
        task.status === TaskStatus.Done ? TaskStatus.Pending : TaskStatus.Done;
      updateTask(id, {status: newStatus});
    },
    [doc, updateTask],
  );

  return {
    createTask,
    updateTask,
    deleteTask,
    moveTask,
    indentTask,
    outdentTask,
    toggleTask,
  };
}
