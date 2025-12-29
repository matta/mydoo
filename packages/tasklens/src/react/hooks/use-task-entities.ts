import {useSelector} from 'react-redux';
import {selectTaskEntities} from '../../store/selectors';
import type {ComputedTask, TaskID} from '../../types';

/**
 * useTaskEntities Hook
 *
 * Provides direct access to the full map of ComputedTasks in the Redux store.
 * Useful for views that need to look up tasks by ID or aggregate over all tasks
 * (like the Balance View).
 *
 * @returns A Record mapping TaskID to ComputedTask.
 */
export function useTaskEntities(): Record<TaskID, ComputedTask> {
  return useSelector(selectTaskEntities);
}
