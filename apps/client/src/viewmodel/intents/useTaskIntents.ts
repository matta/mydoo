import {
  type DocumentHandle,
  type Task,
  type TaskID,
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
  const {ops} = useTunnel(docUrl);

  const toggleTaskCompletion = useCallback(
    (id: TaskID) => {
      ops.toggleDone(id);
    },
    [ops],
  );

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

  return {
    toggleTaskCompletion,
    createTask,
    deleteTask,
    updateTask,
  };
}
