import {
  type DocumentHandle,
  type TaskID,
  TaskStatus,
  useTunnel,
} from '@mydoo/tasklens';
import {useCallback, useMemo} from 'react';

export interface SystemIntents {
  refreshTaskList: () => void;
}

export function useSystemIntents(docUrl: DocumentHandle): SystemIntents {
  const {doc: _doc, change} = useTunnel(docUrl);

  const refreshTaskList = useCallback(() => {
    change(state => {
      const taskIds = Object.keys(state.tasks) as TaskID[];
      for (const taskId of taskIds) {
        const task = state.tasks[taskId];

        if (task && task.status === TaskStatus.Done && !task.isAcknowledged) {
          task.isAcknowledged = true;
        }
      }
    });
  }, [change]);

  return useMemo(
    () => ({
      refreshTaskList,
    }),
    [refreshTaskList],
  );
}
