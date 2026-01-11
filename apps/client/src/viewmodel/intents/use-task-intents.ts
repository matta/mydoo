import {
  selectRootTaskIds,
  selectTaskEntities,
  TaskActions,
  type TaskCreateInput,
  type TaskID,
  TaskStatus,
  type TaskUpdateInput,
} from "@mydoo/tasklens";

import { useCallback } from "react";
import { useAppDispatch, useAppSelector } from "../../store";

/**
 * Hook to manage user intentions for Tasks.
 *
 * This hook acts as a facade over the generic Redux-backed operations,
 * providing domain-specific actions like "createTask" and "toggleTaskCompletion".
 */
export function useTaskIntents() {
  const dispatch = useAppDispatch();

  // We need the data from Redux to perform logic like "indent" and "toggle"
  const tasks = useAppSelector((state) => selectTaskEntities(state));
  const rootTaskIds = useAppSelector((state) => selectRootTaskIds(state));

  const createTask = useCallback(
    (input: TaskCreateInput): TaskID => {
      // Generate ID synchronously to maintain legacy API contract
      const id = crypto.randomUUID() as TaskID;
      // Dispatch thunk (async action) - fire and forget
      dispatch(TaskActions.createTask({ ...input, id }));
      return id;
    },
    [dispatch],
  );

  const updateTask = useCallback(
    (id: TaskID, updates: TaskUpdateInput) => {
      dispatch(TaskActions.updateTask({ id, updates }));
    },
    [dispatch],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      dispatch(TaskActions.deleteTask(id));
    },
    [dispatch],
  );

  const moveTask = useCallback(
    (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => {
      dispatch(TaskActions.moveTask({ id, newParentId, afterTaskId }));
    },
    [dispatch],
  );

  /**
   * Demotes a task to become a child of its previous sibling.
   *
   * @param id - The ID of the task to indent.
   */
  const indentTask = useCallback(
    (id: TaskID) => {
      const task = tasks[id];
      if (!task) return;

      // Determine the list of siblings to find the predecessor
      const siblings = task.parentId
        ? tasks[task.parentId]?.childTaskIds
        : rootTaskIds;

      if (!siblings) return;

      const idx = siblings.indexOf(id);
      // Cannot indent if first child (idx 0) or not found (-1)
      if (idx <= 0) return;

      const prevSiblingId = siblings[idx - 1];
      if (!prevSiblingId) return;

      const prevSibling = tasks[prevSiblingId];
      if (!prevSibling) return;

      // Move to become the last child of the previous sibling
      // We need the last child ID of the new parent to append.
      const newParentLastChildId =
        prevSibling.childTaskIds.length > 0
          ? prevSibling.childTaskIds[prevSibling.childTaskIds.length - 1]
          : undefined;

      dispatch(
        TaskActions.moveTask({
          id,
          newParentId: prevSiblingId,
          afterTaskId: newParentLastChildId,
        }),
      );
    },
    [tasks, rootTaskIds, dispatch],
  );

  /**
   * Promotes a task to be a sibling of its current parent.
   *
   * @param id - The ID of the task to outdent.
   */
  const outdentTask = useCallback(
    (id: TaskID) => {
      const task = tasks[id];
      if (!task) return;

      const parentId = task.parentId;
      if (!parentId) return; // Already at root, cannot outdent

      const parent = tasks[parentId];
      if (!parent) return;

      // Move to be a sibling of the parent, immediately following it.
      dispatch(
        TaskActions.moveTask({
          id,
          newParentId: parent.parentId,
          afterTaskId: parentId,
        }),
      );
    },
    [tasks, dispatch],
  );

  /**
   * Toggles the completion status of a task between 'Done' and 'Pending'.
   * @param id - The ID of the task to toggle.
   */
  const toggleTask = useCallback(
    (id: TaskID) => {
      const task = tasks[id];
      if (!task) return;

      const newStatus =
        task.status === TaskStatus.Done ? TaskStatus.Pending : TaskStatus.Done;
      dispatch(TaskActions.updateTask({ id, updates: { status: newStatus } }));
    },
    [tasks, dispatch],
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
