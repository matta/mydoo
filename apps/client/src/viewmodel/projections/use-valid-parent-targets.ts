import {
  buildTunnelTree,
  type RootState,
  type TaskID,
  type TunnelNode,
} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

export interface ValidTargets {
  isLoading: boolean;
  roots: TunnelNode[];
}

/**
 * Hook to retrieve the task tree filtered for valid reparenting targets.
 *
 * Excludes the task being moved and all its descendants to prevent cycles.
 */
export function useValidParentTargets(
  movedTaskId: TaskID | undefined,
): ValidTargets {
  const doc = useSelector((state: RootState) => state.tasks.lastDoc);

  const roots = useMemo(() => {
    if (!doc || !movedTaskId) {
      if (!doc) return [];
      return buildTunnelTree(doc.rootTaskIds, doc.tasks);
    }

    const fullTree = buildTunnelTree(doc.rootTaskIds, doc.tasks);

    // Recursive filter function
    const filterNode = (node: TunnelNode): TunnelNode | null => {
      // If this is the task being moved, exclude it (and its subtree)
      if (node.id === movedTaskId) {
        return null;
      }

      // Filter children
      const filteredChildren = node.children
        .map(filterNode)
        .filter((n): n is TunnelNode => n !== null);

      // Return new node with filtered children
      return {
        ...node,
        children: filteredChildren,
      };
    };

    return fullTree.map(filterNode).filter((n): n is TunnelNode => n !== null);
  }, [doc, movedTaskId]);

  return {
    roots,
    isLoading: doc === undefined,
  };
}
