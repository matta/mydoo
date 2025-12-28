import type {EnrichedTask, TaskID, TunnelState} from '../../src/types';

/**
 * Pass 7: Container Visibility
 * Recursively hides any Container Task that has at least one visible descendant.
 *
 * @param doc The current Automerge document state.
 * @param tasks All tasks (Mutable EnrichedTasks).
 * @param getChildrenFromMap Helper to get children (EnrichedTasks).
 */
export function pass7ContainerVisibility(
  _doc: TunnelState,
  tasks: EnrichedTask[],
  getChildrenFromMap: (parentId: TaskID | undefined) => EnrichedTask[],
): void {
  // Goal: Ensure the Todo List remains uncluttered, showing only actionable leaves or empty containers.
  // Logic: For each Task T where IsContainer = True: If Any(Descendants(T).Visibility == True): Set T.Visibility = False & T.Priority = 0.0

  // First, identify all container tasks
  const containerTasks = tasks.filter(
    task => getChildrenFromMap(task.id).length > 0,
  );

  // Function to recursively check for visible descendants
  function hasVisibleDescendant(currentTask: EnrichedTask): boolean {
    const children = getChildrenFromMap(currentTask.id);
    if (children.length === 0) {
      // Leaf node, check its own visibility (from previous passes)
      return currentTask.visibility === true;
    }

    // Recurse for children
    return children.some(child => hasVisibleDescendant(child));
  }

  // Iterate through container tasks and apply visibility rule
  for (const container of containerTasks) {
    // Only hide if it has visible descendants
    if (hasVisibleDescendant(container)) {
      container.visibility = false;
      container.priority = 0.0;
    }
  }
}
