import {
  buildTree,
  type DocumentHandle,
  type TunnelNode,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

export interface TaskTree {
  isLoading: boolean;
  roots: TunnelNode[];
}

/**
 * Hook to retrieve the hierarchical task tree.
 *
 * Respects the manual ordering defined in `rootTaskIds` and `childTaskIds`.
 * Returns all tasks regardless of status (view filters should handle visibility).
 */
export function useTaskTree(docUrl: DocumentHandle): TaskTree {
  const {doc} = useTunnel(docUrl);

  const roots = useMemo(() => {
    if (!doc) return [];
    return buildTree(doc.rootTaskIds, doc.tasks);
  }, [doc]);

  return {
    roots,
    isLoading: doc === undefined,
  };
}
