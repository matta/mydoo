import {
  buildTunnelTree,
  type ComputedTask,
  selectRootTaskIds,
  selectStoreReady,
  selectTaskEntities,
  type TaskID,
  type TunnelNode,
} from "@mydoo/tasklens";
import { useMemo } from "react";
import { useSelector } from "react-redux";

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
  const tasks = useSelector(selectTaskEntities);
  const rootTaskIds = useSelector(selectRootTaskIds);
  const isReady = useSelector(selectStoreReady);

  const roots = useMemo(
    () => projectValidTargets(tasks, rootTaskIds, isReady, taskId),
    [tasks, rootTaskIds, isReady, taskId],
  );

  return {
    roots,
    isLoading: !isReady,
  };
}

/**
 * Pure projection function to compute valid parent targets.
 * Factored out to keep useMemo body concise and comply with line limits.
 */
function projectValidTargets(
  tasks: Record<TaskID, ComputedTask>,
  rootTaskIds: TaskID[],
  isReady: boolean,
  taskId: TaskID | undefined,
): TunnelNode[] {
  if (!isReady) return [];

  const fullTree = buildTunnelTree(rootTaskIds, tasks);

  if (!taskId) {
    return fullTree;
  }

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
