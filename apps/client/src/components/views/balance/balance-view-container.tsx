import {Container, Paper, Stack, Text, Title} from '@mantine/core';
import {distributeCredits, type TaskID, useTaskActions} from '@mydoo/tasklens';
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
    const updates = distributeCredits(targetId, newValue, items);
    // TODO: SAFETY CRITICAL: This loop causes multiple Automerge commits for a
    // single user action. This breaks "Undo" functionality (user has to undo 5
    // times to undo 1 drag) and pollutes history. We strictly need a
    // `updateTasks(updates)` action that wraps these in one `doc.change`.
    for (const update of updates) {
      updateTask(update.id, {desiredCredits: update.desiredCredits});
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
