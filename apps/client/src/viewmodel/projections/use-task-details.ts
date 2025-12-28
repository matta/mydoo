import {type RootState, type TaskID, TunnelOps, useTask} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

/**
 * Hook to retrieve full details for a specific task, suitable for the Editor Modal.
 *
 * Provides the task object, the title of its parent, and the total count of descendants.
 */
export function useTaskDetails(taskId: TaskID | undefined) {
  const task = useTask(taskId);
  const doc = useSelector((state: RootState) => state.tasks.lastDoc);

  return useMemo(() => {
    if (!doc) {
      return {
        task: null,
        parentTitle: null,
        descendantCount: 0,
        isLoading: true,
      };
    }

    if (!taskId || !task) {
      return {
        task: null,
        parentTitle: null,
        descendantCount: 0,
        isLoading: false,
      };
    }

    const parentId = task.parentId;
    const parentTask = parentId ? doc.tasks[parentId] : null;
    const parentTitle = parentTask ? parentTask.title : null;

    const descendantCount = TunnelOps.getDescendantCount(doc, taskId);

    return {
      task,
      parentTitle,
      descendantCount,
      isLoading: false,
    };
  }, [doc, task, taskId]);
}
