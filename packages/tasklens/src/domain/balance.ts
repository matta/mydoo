import type {BalanceItemData, ComputedTask} from '../types';

/**
 * The percentage of target credit allocation below which a goal is considered "starving".
 * A value of 0.9 means the goal is starving if it has received less than 90% of its
 * intended attention.
 */
export const STARVING_THRESHOLD = 0.9;

/**
 * Computes the data necessary for the Balance View by aggregating tasks.
 *
 * It calculates the relative percentage of "Desired Credits" (what the user wants
 * to focus on) versus "Effective Credits" (what they have actually worked on)
 * for all root-level tasks (goals).
 */
export function calculateBalanceData(tasks: ComputedTask[]): BalanceItemData[] {
  // Filter for root tasks
  const rootGoals = tasks.filter((task) => task.parentId === undefined);

  let totalDesired = 0;
  let totalActual = 0;

  for (const goal of rootGoals) {
    totalDesired += goal.desiredCredits;
    totalActual += goal.effectiveCredits;
  }

  return rootGoals.map((goal) => {
    const targetPercent =
      totalDesired > 0 ? (goal.desiredCredits / totalDesired) * 100 : 0;
    const actualPercent =
      totalActual > 0 ? (goal.effectiveCredits / totalActual) * 100 : 0;

    // A goal is "starving" if Actual is significantly lower than Target.
    const isStarving =
      targetPercent > 0 && actualPercent < targetPercent * STARVING_THRESHOLD;

    return {
      id: goal.id,
      title: goal.title,
      desiredCredits: goal.desiredCredits,
      effectiveCredits: goal.effectiveCredits,
      targetPercent,
      actualPercent,
      isStarving,
    };
  });
}
