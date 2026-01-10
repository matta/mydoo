import { useSelector } from "react-redux";
import { selectTaskById } from "../../store/selectors";
import type { ComputedTask, TaskID } from "../../types/ui";

/**
 * Hook to access a specific task from the Redux store.
 *
 * This provides a stable reference to a ComputedTask, including its
 * current state and derived properties.
 *
 * @param id - The ID of the task to retrieve.
 * @returns The ComputedTask object, or undefined if not found.
 */
export function useTask(id: TaskID | undefined): ComputedTask | undefined {
  return useSelector(selectTaskById(id));
}
