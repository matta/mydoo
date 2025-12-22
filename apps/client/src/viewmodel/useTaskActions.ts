import {type AnyDocumentId} from '@automerge/automerge-repo';
import {type Task, type TaskID, useTunnel} from '@mydoo/tasklens';
import {useCallback} from 'react';

/**
 * ViewModel hook for task mutations.
 *
 * Exposes methods to modify the task state, wrapping the low-level `ops`
 * from `useTunnel`.
 *
 * @param docUrl - The URL of the Automerge document.
 * @returns Object containing action methods.
 */
export function useTaskActions(docUrl: AnyDocumentId) {
  const {ops} = useTunnel(docUrl);

  const addTask = useCallback(
    (title: string, parentId?: TaskID) => {
      ops.add({title, parentId});
    },
    [ops],
  );

  const updateTask = useCallback(
    (id: TaskID, props: Partial<Task>) => {
      ops.update(id, props);
    },
    [ops],
  );

  const toggleDone = useCallback(
    (id: TaskID) => {
      ops.toggleDone(id);
    },
    [ops],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      ops.delete(id);
    },
    [ops],
  );

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

  return {
    addTask,
    updateTask,
    toggleDone,
    deleteTask,
    moveTask,
  };
}
