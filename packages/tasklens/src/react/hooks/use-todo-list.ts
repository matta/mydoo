import { useSelector } from "react-redux";
import { selectTodoList } from "../../store/selectors";
import type { ComputedTask } from "../../types";

/**
 * Hook to access the prioritized "Do" list from the Redux store.
 *
 * This hook provides a stable, memoized list of tasks that are ready
 * for action, sorted by the prioritization algorithm.
 *
 * @returns Array of ComputedTask objects in the "Do" list.
 */
export function useTodoList(): ComputedTask[] {
  return useSelector(selectTodoList);
}
