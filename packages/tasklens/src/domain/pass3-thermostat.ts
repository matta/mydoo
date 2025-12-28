import type {EnrichedTask, TaskID, TunnelState} from '../../src/types';

/**
 * Pass 3: Deviation Feedback (The "Thermostat")
 * Calculates how far each Root Goal is from its target allocation.
 * Updates the `feedbackFactor` property of each root task.
 *
 * Constants: `k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).
 *
 * @param doc The current Automerge document state (mutable proxy or source of truth).
 * @param tasks All tasks (Mutable EnrichedTasks).
 * @param getTaskFromMap Helper to get an EnrichedTask by ID.
 * @param getChildrenFromMap Helper to get EnrichedTask children.
 */
export function pass3DeviationFeedback(
  _doc: TunnelState,
  tasks: EnrichedTask[],
  _getTaskFromMap: (id: TaskID) => EnrichedTask | undefined,
  _getChildrenFromMap: (parentId: TaskID | undefined) => EnrichedTask[],
): void {
  const k = 2.0; // Sensitivity
  const epsilon = 0.001; // Division by zero protection

  // Filter root tasks from the Enriched list
  const rootGoals = tasks.filter(task => task.parentId === undefined);

  let totalDesiredCredits = 0;
  let totalEffectiveCredits = 0;

  // First pass to calculate EffectiveCredits for all tasks
  // (Assuming EffectiveCredits is already calculated in a previous step or here)
  // The spec says EffectiveCredits is a Computed Property. So it should be calculated.
  // EffectiveCredits = Credits * (0.5 ^ (TimeDelta / HalfLife))
  // For Pass 3, we need the *decayed* history.
  // So, let's update EffectiveCredits here first. HalfLife is 7 Days.
  const halfLifeMillis = 7 * 24 * 60 * 60 * 1000; // 7 days in milliseconds

  for (const task of tasks) {
    const timeDelta = getCurrentTimestamp() - task.creditsTimestamp;
    task.effectiveCredits = task.credits * 0.5 ** (timeDelta / halfLifeMillis);
  }

  for (const root of rootGoals) {
    totalDesiredCredits += root.desiredCredits;
    totalEffectiveCredits += root.effectiveCredits ?? 0; // Use 0 if not calculated
  }

  for (const root of rootGoals) {
    if (totalDesiredCredits === 0) {
      // If no desired credits across all roots, feedback is neutral
      root.feedbackFactor = 1.0;
      continue;
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

    root.feedbackFactor = deviationRatio ** k;
  }
}

// getCurrentTimestamp is not available directly in this file
import {getCurrentTimestamp} from '../../src/utils/time';
