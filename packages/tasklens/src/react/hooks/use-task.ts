import {useSelector} from 'react-redux';
import type {RootState} from '../../store';
import type {ComputedTask, TaskID} from '../../types';

/**
 * Hook to access a specific task from the Redux store.
 *
 * This provides a stable reference to a ComputedTask, including its
 * current state and derived properties.
 *
 * @param id - The ID of the task to retrieve.
 * @returns The ComputedTask object, or null if not found.
 */
export function useTask(id: TaskID | undefined): ComputedTask | null {
  return useSelector((state: RootState) => {
    if (!id) return null;
    return state.tasks.entities[id] || null;
  });
}
