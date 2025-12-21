import * as Automerge from "@automerge/automerge";
import type { Task, TunnelState, TaskID } from "../../src/types";

/**
 * Pass 6: Final Priority
 * Combines all factors into a sortable float.
 * Updates the `priority` property of each task.
 *
 * Formula: `Priority = (Visibility ? 1.0 : 0.0) * NormalizedImportance * Root.FeedbackFactor * LeadTimeFactor`
 *
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param getAncestorsFromDoc Helper to get ancestors from the current document state.
 */
export function pass6FinalPriority(
  doc: Automerge.Doc<TunnelState>,
  tasks: Task[],
  getAncestorsFromDoc: (docState: TunnelState, id: TaskID) => Task[],
): void {
  tasks.forEach((task) => {
    // Get the root task's feedback factor
    const ancestors = getAncestorsFromDoc(doc, task.id);
    const rootTask = ancestors[0] ?? task; // If no ancestors, it's a root itself

    const visibilityFactor = task.visibility ? 1.0 : 0.0;
    const normalizedImportanceFactor = task.normalizedImportance ?? 0.0;
    const feedbackFactor = rootTask.feedbackFactor ?? 1.0; // Default to 1.0 if not calculated
    const leadTimeFactor = task.leadTimeFactor ?? 0.0;

    task.priority =
      visibilityFactor *
      normalizedImportanceFactor *
      feedbackFactor *
      leadTimeFactor;
  });
}
