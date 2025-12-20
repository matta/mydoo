import * as Automerge from "@automerge/automerge";
import { Task, TunnelState, TaskID } from "../../src/types";

/**
 * Pass 7: Container Visibility
 * Recursively hides any Container Task that has at least one visible descendant.
 *
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param getChildrenFromDoc Helper to get children from the current document state.
 */
export function pass7ContainerVisibility(
  doc: Automerge.Doc<TunnelState>,
  tasks: Task[],
  getChildrenFromDoc: (
    docState: TunnelState,
    parentId: TaskID | null,
  ) => Task[],
): void {
  // Goal: Ensure the Todo List remains uncluttered, showing only actionable leaves or empty containers.
  // Logic: For each Task T where IsContainer = True: If Any(Descendants(T).Visibility == True): Set T.Visibility = False & T.Priority = 0.0

  // First, identify all container tasks
  const containerTasks = tasks.filter(
    (task) => getChildrenFromDoc(doc, task.id).length > 0,
  );

  // Function to recursively check for visible descendants
  function hasVisibleDescendant(currentTask: Task): boolean {
    const children = getChildrenFromDoc(doc, currentTask.id);
    if (children.length === 0) {
      // Leaf node, check its own visibility (from previous passes)
      return currentTask.visibility === true;
    }

    // Recurse for children
    return children.some((child) => hasVisibleDescendant(child));
  }

  // Iterate through container tasks and apply visibility rule
  containerTasks.forEach((container) => {
    // Only hide if it has visible descendants
    if (hasVisibleDescendant(container)) {
      container.visibility = false;
      container.priority = 0.0;
    }
  });
}
