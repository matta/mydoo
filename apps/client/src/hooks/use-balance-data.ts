import type {TaskID} from '@mydoo/tasklens';
import {ROOT_INBOX_ID, useTaskEntities} from '@mydoo/tasklens';
import {useMemo} from 'react';

export interface BalanceItemData {
  id: TaskID;
  title: string;
  desiredCredits: number;
  effectiveCredits: number;
  targetPercent: number;
  actualPercent: number;
  isStarving: boolean;
}

/**
 * useBalanceData Hook
 *
 * Computes the data necessary for the Balance View by aggregating all root tasks
 * (excluding the system Inbox).
 *
 * It calculates the relative percentage of "Desired Credits" (what the user wants
 * to focus on) versus "Effective Credits" (what they have actually worked on).
 */
export function useBalanceData(): BalanceItemData[] {
  const entities = useTaskEntities();

  return useMemo(() => {
    const tasks = Object.values(entities);

    // Filter for root tasks, excluding the Inbox
    const rootGoals = tasks.filter(
      task => task.parentId === undefined && task.id !== ROOT_INBOX_ID,
    );

    let totalDesired = 0;
    let totalActual = 0;

    for (const goal of rootGoals) {
      totalDesired += goal.desiredCredits;
      totalActual += goal.effectiveCredits;
    }

    return rootGoals.map(goal => {
      const targetPercent =
        totalDesired > 0 ? (goal.desiredCredits / totalDesired) * 100 : 0;
      const actualPercent =
        totalActual > 0 ? (goal.effectiveCredits / totalActual) * 100 : 0;

      // A goal is "starving" if Actual is significantly lower than Target.
      // We'll use a simple threshold for now.
      const isStarving =
        targetPercent > 0 && actualPercent < targetPercent * 0.9;

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
  }, [entities]);
}
