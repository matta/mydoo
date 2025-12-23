import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {useMemo} from 'react';

import {getListAtPath} from '../lib/todoUtils';

/**
 * ViewModel hook for projecting a specific todo list view.
 *
 * This hook filters the full task tree to return the list of tasks
 * visible at the current navigation path.
 *
 * @param tasks - The full task tree.
 * @param viewPath - The current navigation path (array of TaskIDs).
 * @returns The list of tasks at the path, or undefined if path is invalid.
 */
export function useTodoList(tasks: TunnelNode[], viewPath: TaskID[]) {
  const currentList = useMemo(() => {
    return getListAtPath(tasks, viewPath);
  }, [tasks, viewPath]);

  return {
    currentList,
    isPathValid: currentList !== undefined,
  };
}
