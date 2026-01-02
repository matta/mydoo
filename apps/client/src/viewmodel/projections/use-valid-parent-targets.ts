import {
  buildTunnelTree,
  selectLastProxyDoc,
  type TaskID,
  type TunnelNode,
  type TunnelState,
} from '@mydoo/tasklens';
import {useMemo} from 'react';
import {useSelector} from 'react-redux';

export interface ValidTargets {
  isLoading: boolean;
  roots: TunnelNode[];
}

/**
 * Hook to find all tasks that could validly be the parent of a given task.
 */
export function useValidParentTargets(
  taskId: TaskID | undefined,
): ValidTargets {
  const doc = useSelector(selectLastProxyDoc);

  const roots = useMemo(() => projectValidTargets(doc, taskId), [doc, taskId]);

  return {
    roots,
    isLoading: doc === null,
  };
}

/**
 * Pure projection function to compute valid parent targets.
 * Factored out to keep useMemo body concise and comply with line limits.
 */
function projectValidTargets(
  doc: TunnelState | null,
  taskId: TaskID | undefined,
): TunnelNode[] {
  if (!doc || !taskId) {
    if (!doc) return [];
    return buildTunnelTree(doc.rootTaskIds, doc.tasks);
  }

  const fullTree = buildTunnelTree(doc.rootTaskIds, doc.tasks);

  // Recursive filter function
  const filterNode = (node: TunnelNode): TunnelNode | undefined => {
    // If this is the task being moved, exclude it (and its subtree)
    if (node.id === taskId) {
      return undefined;
    }

    // Filter children
    const filteredChildren = node.children
      .map(filterNode)
      .filter((n): n is TunnelNode => n !== undefined);

    // Return new node with filtered children
    return {
      ...node,
      children: filteredChildren,
    };
  };

  return fullTree
    .map(filterNode)
    .filter((n): n is TunnelNode => n !== undefined);
}
