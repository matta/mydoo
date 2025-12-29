import {Container, Paper, Stack, Text, Title} from '@mantine/core';
import {type TaskID, useTaskActions} from '@mydoo/tasklens';
import {useBalanceData} from '../../../hooks/use-balance-data';
import {BalanceItem} from './balance-item';

export function BalanceViewContainer() {
  const items = useBalanceData();
  const {updateTask} = useTaskActions();

  // 1. Calculate Global Constraints based on N (Total Budget = N)
  // Min = 1% of Total Share (where Total Share = N). So Min = 0.01 * N.
  // Max = Total Budget - ((N - 1) * Min).
  // Example: N=3. Min=0.03. MinOthers=0.06. Max=2.94.
  const n = items.length;
  // Safety: If N=0 (no items), strict values don't matter (list is empty).
  // If N=1, Min=0.01, Max=1 (clamped by budget).
  // In reality if N=1, item is forced to 1.0 anyway.
  const minCredits = Math.max(0.01 * n, 0.01); // Ensure at least 0.01
  const maxCredits = Math.max(n - (n - 1) * minCredits, minCredits);

  const handleDesiredCreditsChange = (targetId: TaskID, newValue: number) => {
    // 1. Identify "The Others"
    const otherItems = items.filter(i => i.id !== targetId);

    // If N=1, force 1.0 (ignore slider).
    if (otherItems.length === 0) {
      if (newValue !== 1.0) {
        updateTask(targetId, {desiredCredits: 1.0});
      }
      return;
    }

    // 2. Validate & Clamp New Value against Min/Max
    // The slider should already visually clamp, but we enforce here too.
    const clampedNewValue = Math.max(
      minCredits,
      Math.min(newValue, maxCredits),
    );

    // 3. Distribution Strategy (Drain/Fill from Surplus/Deficit)
    const targetItem = items.find(i => i.id === targetId);
    const oldValue = targetItem?.desiredCredits ?? 0;

    // Calculate Delta (How much we are CHANGING the target)
    const delta = clampedNewValue - oldValue;

    // Precision clamp: If delta is effectively zero, do nothing
    if (Math.abs(delta) < 0.000001) return;

    const updates: {id: TaskID; val: number}[] = [];
    updates.push({id: targetId, val: clampedNewValue});

    if (delta > 0) {
      // TAKING BUDGET (Target grows)
      // We must remove 'delta' from others without violating their Min.
      // We drain proportionally from their "Surplus" (Current - Min).

      const surpluses = otherItems.map(i => ({
        id: i.id,
        surplus: Math.max(0, i.desiredCredits - minCredits),
      }));

      const totalSurplus = surpluses.reduce((acc, s) => acc + s.surplus, 0);

      // Verification: Can we afford this delta?
      // Since Max constraint ensures NewVal <= Total - Min * (N-1),
      // effectively Sum(Others) >= Min * (N-1).
      // So TotalSurplus >= Delta SHOULD hold true mathematically.

      if (totalSurplus === 0) {
        // Edge case: Everyone else is at floor. We can't increase target.
        // This implies target is already at Max (or near it).
        // Check failed? Clamp target to (Total - Sum(others)).
        // But we already clamped to Max. So strict math says this shouldn't happen unless float errors.
        return;
      }

      for (const item of otherItems) {
        // Find item's surplus
        const s = surpluses.find(x => x.id === item.id)?.surplus ?? 0;
        // Proportion of the "Tax" this item pays
        const tax = delta * (s / totalSurplus);
        updates.push({id: item.id, val: item.desiredCredits - tax});
      }
    } else {
      // GIVING BUDGET (Target shrinks)
      // We must distribute -delta (positive amount) to others.
      // We can distribute based on current size (proportional growth).
      // Since we are Adding, we won't violate Min.
      // Will we violate Max? No, because budget constraint holds.

      const budgetToAdd = -delta;

      // Weight by current desiredCredits.
      const totalWeight = otherItems.reduce(
        (acc, i) => acc + i.desiredCredits,
        0,
      );

      if (totalWeight === 0) {
        // Edge: All others are 0 (shouldn't happen with min=0.01*N, but handled).
        const split = budgetToAdd / otherItems.length;
        for (const item of otherItems) {
          updates.push({id: item.id, val: item.desiredCredits + split});
        }
      } else {
        for (const item of otherItems) {
          const share = budgetToAdd * (item.desiredCredits / totalWeight);
          updates.push({id: item.id, val: item.desiredCredits + share});
        }
      }
    }

    // 5. Apply Updates
    for (const update of updates) {
      updateTask(update.id, {desiredCredits: update.val});
    }
  };

  /**
   * TODO: PERFORMANCE OPTIMIZATION
   * Currently, we perform rebalancing and Automerge document updates on every `onChange` (every pixel drag).
   * This is heavy on the transaction history.
   *
   * Migration Plan:
   * 1. Introduce local "virtual" state (via useState or Redux) in this container.
   * 2. Map `items` from `useBalanceData` to this local state on mount/refresh.
   * 3. Change `handleDesiredCreditsChange` to only update that local state.
   * 4. Add a `commitCreditsUpdate` handler that accepts the final local values.
   * 5. Call `commitCreditsUpdate` from BalanceItem's `onChangeEnd`.
   */
  return (
    <Container size="sm" py="xl">
      <Stack gap="xl">
        <Stack gap={4}>
          <Title order={1}>Life Balance</Title>
          <Text c="dimmed">
            Distribute your "Desired Effort" across your top-level goals. See
            how your actual work compares to your targets.
          </Text>
        </Stack>

        <Stack gap="md">
          {items.map(item => (
            <BalanceItem
              key={item.id}
              item={item}
              onChangeDesiredCredits={handleDesiredCreditsChange}
              min={minCredits}
              max={maxCredits}
              totalItems={n}
            />
          ))}

          {items.length === 0 && (
            <Paper p="xl" withBorder style={{textAlign: 'center'}}>
              <Text c="dimmed">
                No top-level goals found. Create some in the "Plan" view to
                start balancing!
              </Text>
            </Paper>
          )}
        </Stack>
      </Stack>
    </Container>
  );
}
