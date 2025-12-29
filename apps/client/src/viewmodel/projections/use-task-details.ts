import {
  selectLastDoc,
  type Task,
  type TaskID,
  TunnelOps,
  type TunnelState,
  useTask,
} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

/**
 * Result of the useTaskDetails hook.
 */
export interface TaskDetails {
  task: Task | undefined;
  parentTitle: string | undefined;
  descendantCount: number;
  isLoading: boolean;
}

/**
 * Hook to retrieve and compute hierarchical task details.
 *
 * @param taskId - The ID of the task to retrieve details for.
 * @returns An object containing the computed task, its parent's title, and the count of its descendants.
 */
export function useTaskDetails(taskId: TaskID | undefined): TaskDetails {
  const task = useTask(taskId);
  const doc = useSelector(selectLastDoc);

  return useMemo(
    () => projectTaskDetails(doc, task, taskId),
    [doc, task, taskId],
  );
}

/**
 * Pure projection function to compute task details from state.
 * Factored out to keep useMemo body concise.
 */
function projectTaskDetails(
  doc: TunnelState | null,
  task: Task | undefined,
  taskId: TaskID | undefined,
): TaskDetails {
  if (!task || !doc || !taskId) {
    return {
      task: undefined,
      parentTitle: undefined,
      descendantCount: 0,
      isLoading: doc === null,
    };
  }

  const parentTask = task.parentId ? doc.tasks[task.parentId] : undefined;
  const parentTitle = parentTask?.title;
  const descendantCount = TunnelOps.getDescendantCount(doc, taskId);

  return {
    task,
    parentTitle,
    descendantCount,
    isLoading: false,
  };
}
