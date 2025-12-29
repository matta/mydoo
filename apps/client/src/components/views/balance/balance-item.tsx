import {Group, Paper, Progress, Slider, Stack, Text} from '@mantine/core';
import type {TaskID} from '@mydoo/tasklens';
import type {BalanceItemData} from '../../../hooks/use-balance-data';

interface BalanceItemProps {
  item: BalanceItemData;
  onChangeDesiredCredits: (id: TaskID, value: number) => void;
  min: number;
  max: number;
  totalItems: number;
}

/**
 * Formats a value as a percentage of total items for display in a label.
 */
function formatPercentLabel(value: number, total: number): string {
  if (total <= 0) return '0%';
  return `${((value / total) * 100).toFixed(0)}%`;
}

export function BalanceItem({
  item,
  onChangeDesiredCredits,
  min,
  max,
  totalItems,
}: BalanceItemProps) {
  return (
    <Paper withBorder p="md" radius="md" shadow="sm">
      <Stack gap="sm">
        <Group justify="space-between">
          <Text fw={600} size="lg">
            {item.title}
          </Text>
        </Group>

        <Stack gap={4}>
          <Group justify="space-between">
            <Text size="sm" c="dimmed">
              Target Percent: {item.targetPercent.toFixed(1)}%
            </Text>
          </Group>
          <Slider
            value={item.desiredCredits}
            // TODO: Switch this to locally change a "virtual" value in the parent
            // and only call `onChangeDesiredCredits` (which will commit to Automerge) on `onChangeEnd`.
            onChange={val => onChangeDesiredCredits(item.id, val)}
            min={min}
            max={max}
            step={0.01}
            label={val => formatPercentLabel(val, totalItems)}
          />
        </Stack>

        <Stack gap={4}>
          <Group justify="space-between">
            <Text size="sm" c="dimmed">
              Actual Percent: {item.actualPercent.toFixed(1)}%
            </Text>
          </Group>
          <Progress
            value={item.actualPercent}
            color={item.isStarving ? 'red' : 'blue'}
            size="xl"
            radius="xl"
            striped
            animated={item.isStarving}
          />
        </Stack>
      </Stack>
    </Paper>
  );
}
