import * as Automerge from '@automerge/automerge';
import type {Task, TunnelState, TaskID} from '../../src/types';

/**
 * Pass 4: Weight Normalization
 * Propagates importance down the tree.
 * Updates the `normalizedImportance` property of each task.
 *
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param getChildrenFromDoc Helper to get children from the current document state.
 */
export function pass4WeightNormalization(
  doc: Automerge.Doc<TunnelState>,
  tasks: Task[],
  getChildrenFromDoc: (
    docState: TunnelState,
    parentId: TaskID | null,
  ) => Task[],
): void {
  const rootTasks = tasks.filter(task => task.parentId === null);

  // Set NormalizedImportance for root tasks
  rootTasks.forEach(root => {
    root.normalizedImportance = 1.0; // Root Goals compete via Feedback, not Weight.
  });

  // Recursively calculate normalizedImportance for children
  function calculateChildImportance(parent: Task) {
    const children = getChildrenFromDoc(doc, parent.id);
    if (children.length === 0) {
      return;
    }

    // Sum of siblings' importance
    const sumSiblingsImportance = children.reduce(
      (sum, child) => sum + child.importance,
      0,
    );

    children.forEach(child => {
      // Avoid division by zero if all children have 0 importance, though importance is 0.0-1.0
      const childImportance = child.importance; // Default to 1.0 if not set
      const parentNormalizedImportance = parent.normalizedImportance ?? 0;
      if (sumSiblingsImportance === 0) {
        // If sum is 0 (all children have 0 importance), distribute equally or handle as error
        // For now, distribute equally if sum is 0
        child.normalizedImportance =
          parentNormalizedImportance / children.length;
      } else {
        child.normalizedImportance =
          (childImportance / sumSiblingsImportance) *
          parentNormalizedImportance;
      }
      calculateChildImportance(child); // Recurse for children
    });
  }

  // Start recursion from root tasks
  rootTasks.forEach(root => {
    calculateChildImportance(root);
  });
}
