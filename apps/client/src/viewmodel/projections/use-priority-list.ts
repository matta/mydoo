import { selectStoreReady } from "@mydoo/tasklens";
import { useAppSelector } from "../../store";
import { useTodoList } from "../hooks/task-hooks";

/**
 * Hook to retrieve a prioritized list of pending tasks.
 *
 * Uses the Redux store for stable, memoized access.
 */
export function usePriorityList() {
  const tasks = useTodoList();
  const isReady = useAppSelector(selectStoreReady);

  return {
    tasks,
    isLoading: !isReady,
  };
}
