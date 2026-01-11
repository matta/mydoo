import {
  type ComputedTask,
  selectTaskEntities,
  selectTodoList,
  type TaskID,
} from "@mydoo/tasklens";
import { useAppSelector } from "../../store";

/**
 * Hook to retrieve a specific task by ID.
 * Moved from tasklens package to client viewmodel.
 */
export function useTask(id: TaskID | undefined) {
  const entities = useAppSelector(selectTaskEntities);
  if (!id) return undefined;
  return entities[id];
}

/**
 * Hook to access the prioritized "Do" list from the Redux store.
 */
export function useTodoList(): ComputedTask[] {
  return useAppSelector(selectTodoList);
}
