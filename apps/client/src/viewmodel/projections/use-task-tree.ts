import {
  buildTunnelTree,
  ROOT_INBOX_ID,
  selectLastDoc,
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
  const doc = useSelector(selectLastDoc);

  const roots = useMemo(() => {
    if (!doc) return [];
    // Plan view excludes the system Inbox
    const filteredRootIds = doc.rootTaskIds.filter(id => id !== ROOT_INBOX_ID);
    return buildTunnelTree(filteredRootIds, doc.tasks);
  }, [doc]);

  return {
    roots,
    isLoading: doc === null,
  };
}
