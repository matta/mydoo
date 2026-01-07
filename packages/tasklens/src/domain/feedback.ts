import type { EnrichedTask } from '../types';

/**
 * Adaptive Feedback Control
 * Calculates how far each Root Goal is from its target allocation.
 * Updates the `feedbackFactor` property of each root task.
 *
 * Constants: `k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).
 *
 * @param tasks All tasks (Mutable EnrichedTasks).
 */
export function calculateFeedbackFactors(tasks: EnrichedTask[]): void {
  const k = 2.0; // Sensitivity
  const epsilon = 0.001; // Division by zero protection

  // Filter root tasks from the Enriched list
  const rootGoals = tasks.filter((task) => task.parentId === undefined);

  let totalDesiredCredits = 0;
  let totalEffectiveCredits = 0;

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
