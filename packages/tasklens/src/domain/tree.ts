import type {Task, TaskID, TunnelNode} from '../types';

/**
 * Recursive function to build the tree from state.
 *
 * @param taskIds - Ordered list of IDs to resolve (roots or children)
 * @param tasks - Map of all available tasks
 * @returns Array of TunnelNodes with children resolved
 */
export function buildTree(
  taskIds: TaskID[],
  tasks: Record<TaskID, Task>,
): TunnelNode[] {
  return taskIds
    .map(id => tasks[id])
    .filter((t): t is Task => t !== undefined)
    .map(task => ({
      ...task,
      children: buildTree(task.childTaskIds || [], tasks),
    }));
}
