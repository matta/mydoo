import {useSelector} from 'react-redux';
import type {RootState} from '../../store';
import type {ComputedTask} from '../../types';

/**
 * Hook to access the prioritized "Do" list from the Redux store.
 *
 * This hook provides a stable, memoized list of tasks that are ready
 * for action, sorted by the prioritization algorithm.
 *
 * @returns Array of ComputedTask objects in the "Do" list.
 */
export function useTodoList(): ComputedTask[] {
  return useSelector((state: RootState) => {
    const {todoListIds, entities} = state.tasks;
    return todoListIds
      .map(id => entities[id])
      .filter((task): task is ComputedTask => !!task);
  });
}
