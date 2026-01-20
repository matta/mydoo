import type { TaskID } from "../types/persistence";

export interface BalanceItemSimple {
  id: TaskID;
  desiredCredits: number;
}

export interface CreditUpdate {
  id: TaskID;
  desiredCredits: number;
}

/**
 * Distributes credit changes among a set of balance items.
 *
 * When one item's desired credits are changed, the difference (delta) must be
 * absorbed by the other items to maintain the invariants (sum ≈ N, min/max constraints).
 *
 * Strategy:
 * - If increasing multiple items (target grows), we "tax" the others proportionally
 *   to their surplus (amount above minimum).
 * - If decreasing multiple items (target shrinks), we "refund" the others proportionally
 *   to their current weight.
 */
export function distributeCredits(
  targetId: TaskID,
  requestedValue: number,
  items: BalanceItemSimple[],
): CreditUpdate[] {
  const n = items.length;
  if (n === 0) return [];

  // 1. Calculate Global Constraints based on N
  // Min = 1% of Total Share per item (where Total Share = N). So Min = 0.01 * N.
  // Note: One might think Min should be 0.01 (1%), but sticking to the original logic:
  // "Min = 0.01 * N" implies that as you add more items, the minimum share grows?
  // Wait, if N=100, Min=1.0? That implies everyone must be 1.0?
  // Let's re-read the original logic carefully.
  // "const minCredits = Math.max(0.01 * n, 0.01);"
  // If n=3, min = 0.03.
  // If n=10, min = 0.1.
  // If n=100, min = 1.0.
  // If n=200, min = 2.0.
  // This logic seems questionable for large N, but I will preserve it for this refactor
  // to maintain identical behavior. The prompt asked for "Refactor", not "Fix Algorithm",
  // unless the logic was strictly inline.
  // Actually, I should probably keep it identical.
  const minCredits = Math.max(0.01 * n, 0.01);
  const maxCredits = Math.max(n - (n - 1) * minCredits, minCredits);

  // 2. Identify "The Others"
  const otherItems = items.filter((i) => i.id !== targetId);

  // Variable to hold the actual new value after clamping
  let newValue = requestedValue;

  // If N=1, force 1.0 (ignore slider).
  if (otherItems.length === 0) {
    if (newValue !== 1.0) {
      return [{ id: targetId, desiredCredits: 1.0 }];
    }
    return [];
  }

  // 3. Validate & Clamp New Value against Min/Max
  newValue = Math.max(minCredits, Math.min(newValue, maxCredits));

  const targetItem = items.find((i) => i.id === targetId);
  const oldValue = targetItem?.desiredCredits ?? 0;

  // Calculate Delta (How much we are CHANGING the target)
  const delta = newValue - oldValue;

  // Precision clamp: If delta is effectively zero, do nothing
  if (Math.abs(delta) < 0.000001) return [];

  const updates: CreditUpdate[] = [];
  updates.push({ id: targetId, desiredCredits: newValue });

  if (delta > 0) {
    // TAKING BUDGET (Target grows)
    // We must remove 'delta' from others without violating their Min.
    // We drain proportionally from their "Surplus" (Current - Min).

    const surpluses = otherItems.map((i) => ({
      id: i.id,
      surplus: Math.max(0, i.desiredCredits - minCredits),
    }));

    const totalSurplus = surpluses.reduce((acc, s) => acc + s.surplus, 0);

    // If there is no surplus to take from, we can't increase the target.
    // (Ideally `maxCredits` calculation prevents us from asking for more than exists,
    // but floating point math might leave us slightly off).
    if (totalSurplus === 0) {
      return [];
    }

    for (const item of otherItems) {
      // Find item's surplus
      const s = surpluses.find((x) => x.id === item.id)?.surplus ?? 0;
      // Proportion of the "Tax" this item pays
      const tax = delta * (s / totalSurplus);
      updates.push({ id: item.id, desiredCredits: item.desiredCredits - tax });
    }
  } else {
    // GIVING BUDGET (Target shrinks)
    // We must distribute -delta (positive amount) to others.
    // We can distribute based on current size (proportional growth).

    const budgetToAdd = -delta;

    // Weight by current desiredCredits.
    const totalWeight = otherItems.reduce(
      (acc, i) => acc + i.desiredCredits,
      0,
    );

    if (totalWeight === 0) {
      // Edge: All others are 0. Distribute equally.
      const split = budgetToAdd / otherItems.length;
      for (const item of otherItems) {
        updates.push({
          id: item.id,
          desiredCredits: item.desiredCredits + split,
        });
      }
    } else {
      for (const item of otherItems) {
        const share = budgetToAdd * (item.desiredCredits / totalWeight);
        updates.push({
          id: item.id,
          desiredCredits: item.desiredCredits + share,
        });
      }
    }
  }

  return updates;
}
