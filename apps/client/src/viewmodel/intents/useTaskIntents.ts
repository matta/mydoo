import {
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
    (title: string, parentId?: TaskID) => {
      ops.add({title, parentId});
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
    toggleTask,
  };
}
