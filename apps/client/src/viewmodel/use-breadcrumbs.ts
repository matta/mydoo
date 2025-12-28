import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {useMemo} from 'react';

import {getBreadcrumbs} from '../lib/todo-utils';

/**
 * ViewModel hook for generating breadcrumb navigation.
 *
 * @param tasks - The full task tree.
 * @param viewPath - The current navigation path.
 * @returns Array of breadcrumb items suitable for rendering.
 */
export function useBreadcrumbs(tasks: TunnelNode[], viewPath: TaskID[]) {
  const breadcrumbs = useMemo(() => {
    return getBreadcrumbs(tasks, viewPath);
  }, [tasks, viewPath]);

  return breadcrumbs;
}
