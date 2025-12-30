import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import {useCallback} from 'react';
import {
  createTask as createTaskOp,
  deleteTask as deleteTaskOp,
  moveTask as moveTaskOp,
  updateTask as updateTaskOp,
} from '../../persistence/ops';
import type {CreateTaskOptions, Task, TaskID, TunnelState} from '../../types';
import {TaskStatus} from '../../types';
import {useTaskLensDocId} from '../task-lens-provider';

/**
 * Hook to perform mutations on the TaskLens document.
 *
 * This hook provides type-safe methods for creating, updating,
 * and moving tasks. It automatically handles the Automerge
 * transaction lifecycle.
 */
export function useTaskActions() {
  const docId = useTaskLensDocId();
  // biome-ignore lint/suspicious/noExplicitAny: internal handle type erasure
  const handle = useDocHandle<TunnelState>(docId as any);

  const mutate = useCallback(
    (callback: (d: TunnelState) => void) => {
      if (!handle) return;
      handle.change(d => {
        callback(d as TunnelState);
      });
    },
    [handle],
  );

  const createTask = useCallback(
    (
      title: string,
      parentId?: TaskID,
      options?: CreateTaskOptions,
      props?: Partial<Task>,
    ): TaskID => {
      const newTaskId = crypto.randomUUID() as TaskID;
      mutate(d => {
        createTaskOp(d, {id: newTaskId, title, parentId, ...props}, options);
      });
      return newTaskId;
    },
    [mutate],
  );

  const updateTask = useCallback(
    (id: TaskID, updates: Partial<Task>) => {
      mutate(d => {
        updateTaskOp(d, id, updates);
      });
    },
    [mutate],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      mutate(d => {
        deleteTaskOp(d, id);
      });
    },
    [mutate],
  );

  const moveTask = useCallback(
    (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => {
      mutate(d => {
        moveTaskOp(d, id, newParentId, afterTaskId);
      });
    },
    [mutate],
  );

  const acknowledgeAllDoneTasks = useCallback(() => {
    mutate(doc => {
      const taskIds = Object.keys(doc.tasks) as TaskID[];
      for (const taskId of taskIds) {
        const task = doc.tasks[taskId];
        if (task && task.status === TaskStatus.Done && !task.isAcknowledged) {
          task.isAcknowledged = true;
        }
      }
    });
  }, [mutate]);

  return {
    createTask,
    updateTask,
    deleteTask,
    moveTask,
    acknowledgeAllDoneTasks,
  };
}
