import {
  type DocumentHandle,
  type TaskID,
  TunnelOps,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

/**
 * Hook to retrieve full details for a specific task, suitable for the Editor Modal.
 *
 * Provides the task object, the title of its parent, and the total count of descendants.
 */
export function useTaskDetails(docUrl: DocumentHandle, taskId: TaskID) {
  const {doc} = useTunnel(docUrl);

  return useMemo(() => {
    if (!doc) {
      return {
        task: null,
        parentTitle: null,
        descendantCount: 0,
        isLoading: true,
      };
    }

    const task = doc.tasks[taskId] || null;
    const parentId = task?.parentId;
    const parentTask = parentId ? doc.tasks[parentId] : null;
    const parentTitle = parentTask ? parentTask.title : null;

    const descendantCount = task
      ? TunnelOps.getDescendantCount(doc, taskId)
      : 0;

    return {
      task,
      parentTitle,
      descendantCount,
      isLoading: false,
    };
  }, [doc, taskId]);
}
