import {
  type PersistedTask,
  selectLastProxyDoc,
  type TaskID,
} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

/**
 * Represents a single segment in the navigation breadcrumb trail.
 */
export interface BreadcrumbItem {
  id: TaskID;
  title: string;
}

/**
 * Derives the navigation path from the root to the specified task.
 *
 * @param currentViewId - The TaskID of the currently focused task, or undefined if viewing the root.
 * @returns An ordered array of `BreadcrumbItem`s representing the ancestry path.
 */
export function useBreadcrumbs(
  currentViewId: TaskID | undefined,
): BreadcrumbItem[] {
  const doc = useSelector(selectLastProxyDoc);

  const breadcrumbs = useMemo(() => {
    if (!doc || !currentViewId) return [];

    const path: BreadcrumbItem[] = [];
    let currentId: TaskID | undefined = currentViewId;

    // Safety limit to prevent infinite loops in cases of graph cycles.
    let safetyCounter = 0;
    const MAX_DEPTH = 50;

    while (currentId && safetyCounter < MAX_DEPTH) {
      if (!doc) break; // Defensive check
      const task: PersistedTask | undefined = doc.tasks[currentId];
      if (!task) break;

      path.unshift({
        id: task.id,
        title: task.title,
      });

      currentId = task.parentId;
      safetyCounter++;
    }

    return path;
  }, [doc, currentViewId]);

  return breadcrumbs;
}
