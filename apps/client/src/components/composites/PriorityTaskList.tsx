import {Stack, Text} from '@mantine/core';
import type {Task, TaskID} from '@mydoo/tasklens';

import {TaskRow} from '../primitives/TaskRow';

export interface PriorityTaskListProps {
  onToggle: (id: TaskID) => void;
  tasks: Task[];
}

export function PriorityTaskList({tasks, onToggle}: PriorityTaskListProps) {
  if (tasks.length === 0) {
    return (
      <Text c="dimmed" py="xl" ta="center">
        No tasks needed. Great job!
      </Text>
    );
  }

  return (
    <Stack gap="xs">
      {tasks.map(task => (
        <TaskRow key={task.id} onToggle={onToggle} task={task} />
      ))}
    </Stack>
  );
}
