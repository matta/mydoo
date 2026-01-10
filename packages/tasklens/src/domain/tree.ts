import type { TaskID } from "../types/persistence";
import type { TaskFields, TunnelNode } from "../types/ui";
import { toComputedTask } from "./projections";

/**
 * Builds the complete TunnelNode tree with computed properties in a single pass.
 *
 * This is the idiomatic way to get a UI-ready tree from TunnelState.
 *
 * @param rootTaskIds - The root task IDs (e.g., doc.rootTaskIds)
 * @param tasks - The tasks map (e.g., doc.tasks)
 * @returns Array of TunnelNodes with isReady, isContainer, isPending computed
 */
export function buildTunnelTree<T extends TaskFields>(
  rootTaskIds: TaskID[],
  tasks: Record<TaskID, T>,
): TunnelNode[] {
  const build = (taskIds: TaskID[]): TunnelNode[] =>
    taskIds
      .map((id) => tasks[id])
      .filter((t): t is T => t !== undefined)
      .map((task) => ({
        ...toComputedTask(task),
        children: build(task.childTaskIds || []),
      }));

  return build(rootTaskIds);
}
