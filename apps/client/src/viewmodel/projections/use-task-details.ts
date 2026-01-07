import {
  type ComputedTask,
  getDescendantCountFromEntities,
  selectStoreReady,
  selectTaskEntities,
  type Task,
  type TaskID,
  useTask,
} from "@mydoo/tasklens";
import { useMemo } from "react";
import { useSelector } from "react-redux";

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
  const tasks = useSelector(selectTaskEntities);
  const isReady = useSelector(selectStoreReady);

  return useMemo(
    () => projectTaskDetails(tasks, task, taskId, isReady),
    [tasks, task, taskId, isReady],
  );
}

/**
 * Pure projection function to compute task details from state.
 * Factored out to keep useMemo body concise.
 */
function projectTaskDetails(
  tasks: Record<TaskID, ComputedTask>,
  task: Task | undefined,
  taskId: TaskID | undefined,
  isReady: boolean,
): TaskDetails {
  if (!isReady) {
    return {
      task: undefined,
      parentTitle: undefined,
      descendantCount: 0,
      isLoading: true,
    };
  }

  if (!task || !taskId) {
    return {
      task: undefined,
      parentTitle: undefined,
      descendantCount: 0,
      isLoading: false,
    };
  }

  const parentTask = task.parentId ? tasks[task.parentId] : undefined;
  const parentTitle = parentTask?.title;
  const descendantCount = getDescendantCountFromEntities(tasks, taskId);

  return {
    task,
    parentTitle,
    descendantCount,
    isLoading: false,
  };
}
