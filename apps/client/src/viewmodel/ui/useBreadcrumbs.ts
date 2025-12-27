import type {
  DocumentHandle,
  PersistedTask,
  TaskID,
  TunnelState,
} from '@mydoo/tasklens';
import {useTunnel} from '@mydoo/tasklens';
import {useMemo} from 'react';

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
 * @param docUrl - The DocumentHandle for the active Automerge document.
 * @param currentViewId - The TaskID of the currently focused task, or undefined if viewing the root.
 * @returns An ordered array of `BreadcrumbItem`s representing the ancestry path.
 *
 * @remarks
 * - The returned path excludes the logic checks for root/home; it purely represents the specific
 *   task hierarchy leading to `currentViewId`.
 * - Includes a safety mechanism to halt traversal if the depth exceeds `MAX_DEPTH` (50),
 *   preventing infinite loops in the event of cyclic data structures.
 */
export function useBreadcrumbs(
  docUrl: DocumentHandle,
  currentViewId: TaskID | undefined,
): BreadcrumbItem[] {
  const {doc}: {doc: TunnelState | undefined} = useTunnel(docUrl);

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
