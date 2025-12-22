import {type TaskID, type TunnelNode} from '@mydoo/tasklens';
import {useMemo} from 'react';

import {getBreadcrumbs} from '../lib/todoUtils';

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
