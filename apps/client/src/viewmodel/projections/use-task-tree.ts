import {
  buildTunnelTree,
  type RootState,
  type TunnelNode,
} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

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
export function useTaskTree(): TaskTree {
  const doc = useSelector((state: RootState) => state.tasks.lastDoc);

  const roots = useMemo(() => {
    if (!doc) return [];
    return buildTunnelTree(doc.rootTaskIds, doc.tasks);
  }, [doc]);

  return {
    roots,
    isLoading: doc === undefined,
  };
}
