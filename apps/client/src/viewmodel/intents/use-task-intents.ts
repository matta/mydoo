import {
  type CreateTaskOptions,
  selectLastDoc,
  type Task,
  type TaskID,
  TaskStatus,
  useTaskActions,
} from '@mydoo/tasklens';
import {useCallback} from 'react';
import {useSelector} from 'react-redux';

/**
 * Hook to manage user intentions for Tasks.
 *
 * This hook acts as a facade over the generic Redux-backed operations,
 * providing domain-specific actions like "createTask" and "toggleTaskCompletion".
 */
export function useTaskIntents() {
  const {
    createTask: baseCreateTask,
    updateTask,
    deleteTask,
    moveTask,
  } = useTaskActions();

  // We need the data from Redux to perform logic like "indent" and "toggle"
  const doc = useSelector(selectLastDoc);

  const createTask = useCallback(
    (
      title: string,
      parentId?: TaskID,
      options?: CreateTaskOptions,
      props?: Partial<Task>,
    ): TaskID => {
      return baseCreateTask(title, parentId, options, props);
    },
    [baseCreateTask],
  );

  /**
   * Demotes a task to become a child of its previous sibling.
   *
   * @param id - The ID of the task to indent.
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

      moveTask(id, prevSiblingId, newParentLastChildId);
    },
    [doc, moveTask],
  );

  /**
   * Promotes a task to be a sibling of its current parent.
   *
   * @param id - The ID of the task to outdent.
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
      moveTask(id, parent.parentId, parentId);
    },
    [doc, moveTask],
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
