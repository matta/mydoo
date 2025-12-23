import {ActionIcon, Checkbox, Group, Paper, Text} from '@mantine/core';
import type {Task, TaskID} from '@mydoo/tasklens';
import {IconGripVertical} from '@tabler/icons-react';

export interface TaskRowProps {
  onToggle: (id: TaskID) => void;
  style?: React.CSSProperties;
  task: Task;
}

export function TaskRow({task, onToggle, style}: TaskRowProps) {
  return (
    <Paper p="xs" shadow="xs" style={style} withBorder>
      <Group align="center" gap="sm" wrap="nowrap">
        <ActionIcon color="gray" style={{cursor: 'grab'}} variant="transparent">
          <IconGripVertical size={16} />
        </ActionIcon>

        <Checkbox
          aria-label={`Complete ${task.title}`}
          checked={task.status === 'Done'}
          onChange={() => {
            onToggle(task.id);
          }}
        />

        <Text
          c={task.status === 'Done' ? 'dimmed' : ''}
          flex={1}
          style={{
            userSelect: 'none',

            textDecoration: task.status === 'Done' ? 'line-through' : undefined,
          }}
        >
          {task.title}
        </Text>

        {/* Placeholder for future metadata/importance */}
        <Text c="dimmed" size="xs">
          {task.importance.toFixed(2)}
        </Text>
      </Group>
    </Paper>
  );
}
