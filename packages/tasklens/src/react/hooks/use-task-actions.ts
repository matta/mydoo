import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import {useCallback} from 'react';
import * as TunnelOps from '../../persistence/ops';
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
        TunnelOps.createTask(
          d,
          {id: newTaskId, title, parentId, ...props},
          options,
        );
      });
      return newTaskId;
    },
    [mutate],
  );

  const updateTask = useCallback(
    (id: TaskID, updates: Partial<Task>) => {
      mutate(d => {
        TunnelOps.updateTask(d, id, updates);
      });
    },
    [mutate],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      mutate(d => {
        TunnelOps.deleteTask(d, id);
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
        TunnelOps.moveTask(d, id, newParentId, afterTaskId);
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
