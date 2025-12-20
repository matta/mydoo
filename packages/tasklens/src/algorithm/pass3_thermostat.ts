import * as Automerge from "@automerge/automerge";
import { Task, TunnelState, TaskID } from "../../src/types";

/**
 * Pass 3: Deviation Feedback (The "Thermostat")
 * Calculates how far each Root Goal is from its target allocation.
 * Updates the `feedbackFactor` property of each root task.
 *
 * Constants: `k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).
 *
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param getTaskFromDoc Helper to get a task from the current document state.
 * @param getChildrenFromDoc Helper to get children from the current document state.
 */
export function pass3DeviationFeedback(
  doc: Automerge.Doc<TunnelState>,
  tasks: Task[],
  _getTaskFromDoc: (docState: TunnelState, id: TaskID) => Task | undefined,
  _getChildrenFromDoc: (
    docState: TunnelState,
    parentId: TaskID | null,
  ) => Task[],
): void {
  const k = 2.0; // Sensitivity
  const epsilon = 0.001; // Division by zero protection

  const rootGoals = tasks.filter((task) => task.parentId === null);

  let totalDesiredCredits = 0;
  let totalEffectiveCredits = 0;

  // First pass to calculate EffectiveCredits for all tasks
  // (Assuming EffectiveCredits is already calculated in a previous step or here)
  // The spec says EffectiveCredits is a Computed Property. So it should be calculated.
  // EffectiveCredits = Credits * (0.5 ^ (TimeDelta / HalfLife))
  // For Pass 3, we need the *decayed* history.
  // So, let's update EffectiveCredits here first. HalfLife is 7 Days.
  const halfLifeMillis = 7 * 24 * 60 * 60 * 1000; // 7 days in milliseconds

  for (const taskId in doc.tasks) {
    const task = doc.tasks[taskId];
    const timeDelta = getCurrentTimestamp() - task.creditsTimestamp;
    task.effectiveCredits =
      task.credits * Math.pow(0.5, timeDelta / halfLifeMillis);
  }

  rootGoals.forEach((root) => {
    totalDesiredCredits += root.desiredCredits;
    totalEffectiveCredits += root.effectiveCredits ?? 0; // Use 0 if not calculated
  });

  rootGoals.forEach((root) => {
    if (totalDesiredCredits === 0) {
      // If no desired credits across all roots, feedback is neutral
      root.feedbackFactor = 1.0;
      return;
    }

    const targetPercent = root.desiredCredits / totalDesiredCredits;
    // Ensure ActualPercent is never lower than epsilon for calculation stability
    const actualPercent =
      (root.effectiveCredits ?? 0) /
      Math.max(totalEffectiveCredits, epsilon * totalDesiredCredits); // Normalize by totalDesired to make epsilon meaningful

    let deviationRatio: number;
    if (targetPercent === 0) {
      deviationRatio = 1.0; // No target, no deviation
    } else {
      deviationRatio = targetPercent / Math.max(actualPercent, epsilon);
    }

    // Cap DeviationRatio to prevent extreme spikes (spec says 1000.0)
    deviationRatio = Math.min(deviationRatio, 1000.0);

    root.feedbackFactor = Math.pow(deviationRatio, k);
  });
}

// getCurrentTimestamp is not available directly in this file
import { getCurrentTimestamp } from "../../src/utils/time";
