import {type DocumentHandle, type TaskID, useTunnel} from '@mydoo/tasklens';
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
    (title: string) => {
      ops.add({title});
    },
    [ops],
  );

  return {
    toggleTaskCompletion,
    createTask,
  };
}
