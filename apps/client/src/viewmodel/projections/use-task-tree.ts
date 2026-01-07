import {
  buildTunnelTree,
  selectRootTaskIds,
  selectStoreReady,
  selectTaskEntities,
  type TunnelNode,
} from '@mydoo/tasklens';
import { useMemo } from 'react';
import { useSelector } from 'react-redux';

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
  const tasks = useSelector(selectTaskEntities);
  const rootTaskIds = useSelector(selectRootTaskIds);
  const isReady = useSelector(selectStoreReady);

  const roots = useMemo(() => {
    if (!isReady) return [];
    return buildTunnelTree(rootTaskIds, tasks);
  }, [isReady, rootTaskIds, tasks]);

  return {
    roots,
    isLoading: !isReady,
  };
}
