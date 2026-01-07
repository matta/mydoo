import { useTasksStatus, useTodoList } from "@mydoo/tasklens";

/**
 * Hook to retrieve a prioritized list of pending tasks.
 *
 * Uses the Redux store populated by TaskLensProvider for stable, memoized access.
 */
export function usePriorityList() {
  const tasks = useTodoList();
  const { isReady } = useTasksStatus();

  return {
    tasks,
    isLoading: !isReady,
  };
}
