import type {EnrichedTask, TaskID, TunnelState} from '../../src/types';

/**
 * Pass 4: Weight Normalization
 * Propagates importance down the tree.
 * Updates the `normalizedImportance` property of each task.
 *
 * @param doc The current Automerge document state (keep signature, though unused if we use tasks access).
 * @param tasks All tasks (Mutable EnrichedTasks).
 * @param getChildrenFromMap Helper to get EnrichedTask children.
 */
export function pass4WeightNormalization(
  _doc: TunnelState,
  tasks: EnrichedTask[],
  getChildrenFromMap: (parentId: TaskID | undefined) => EnrichedTask[],
): void {
  const rootTasks = tasks.filter(task => task.parentId === undefined);

  // Set NormalizedImportance for root tasks
  for (const root of rootTasks) {
    root.normalizedImportance = 1.0; // Root Goals compete via Feedback, not Weight.
  }

  // Recursively calculate normalizedImportance for children
  function calculateChildImportance(parent: EnrichedTask) {
    const children = getChildrenFromMap(parent.id);
    if (children.length === 0) {
      return;
    }

    // Sum of siblings' importance
    const sumSiblingsImportance = children.reduce(
      (sum, child) => sum + child.importance,
      0,
    );

    for (const child of children) {
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
    }
  }

  // Start recursion from root tasks
  for (const root of rootTasks) {
    calculateChildImportance(root);
  }
}
