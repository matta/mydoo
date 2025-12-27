import {
  buildTunnelTree,
  type DocumentHandle,
  type TaskID,
  type TunnelNode,
  useTunnel,
} from '@mydoo/tasklens';
import {useMemo} from 'react';

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
  docUrl: DocumentHandle,
  movedTaskId: TaskID | undefined,
): ValidTargets {
  const {doc} = useTunnel(docUrl);

  const roots = useMemo(() => {
    if (!doc || !movedTaskId) {
      if (!doc) return [];
      // If no task is being moved, technically all are valid, but usually we call this with a task.
      // If movedTaskId is undefined, just return full tree or empty?
      // Let's return full tree if no movedTaskId provided (though unlikely case for this hook).
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
      // We must create a new object to avoid mutating the original tree memo
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
