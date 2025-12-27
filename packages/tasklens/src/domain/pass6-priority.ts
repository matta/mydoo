import type {EnrichedTask, TaskID, TunnelState} from '../../src/types';

/**
 * Pass 6: Final Priority
 * Combines all factors into a sortable float.
 * Updates the `priority` property of each task.
 *
 * Formula: `Priority = (Visibility ? 1.0 : 0.0) * NormalizedImportance * Root.FeedbackFactor * LeadTimeFactor`
 *
 * @param doc The current Automerge document state.
 * @param tasks All tasks (Mutable EnrichedTasks).
 * @param getAncestorsFromMap Helper to get ancestors (EnrichedTasks).
 */
export function pass6FinalPriority(
  _doc: TunnelState,
  tasks: EnrichedTask[],
  getAncestorsFromMap: (id: TaskID) => EnrichedTask[],
): void {
  tasks.forEach(task => {
    const ancestors = getAncestorsFromMap(task.id);
    const rootTask = ancestors[0] ?? task;

    const visibilityFactor = task.visibility ? 1.0 : 0.0;
    const normalizedImportanceFactor = task.normalizedImportance ?? 0.0;
    const feedbackFactor = rootTask.feedbackFactor ?? 1.0;
    const leadTimeFactor = task.leadTimeFactor ?? 0.0;

    task.priority =
      visibilityFactor *
      normalizedImportanceFactor *
      feedbackFactor *
      leadTimeFactor;
  });
}
