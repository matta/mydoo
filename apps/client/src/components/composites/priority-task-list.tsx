import {Stack, Text} from '@mantine/core';
import type {Task, TaskID} from '@mydoo/tasklens';

import {TaskRow} from '../primitives/task-row';

/**
 * Props for the PriorityTaskList component.
 */
export interface PriorityTaskListProps {
  /**
   * Callback fired when a task's completion status is toggled.
   * @param id - The ID of the task being toggled.
   */
  onToggle: (id: TaskID) => void;

  /**
   * Optional callback fired when a task's title is tapped/clicked.
   * Used to open the TaskEditorModal for editing task details.
   * @param id - The ID of the task that was tapped.
   */
  onTitleTap?: (id: TaskID) => void;

  /**
   * The list of tasks to display, sorted by priority.
   * Tasks are rendered as TaskRow components in order.
   */
  tasks: Task[];
}

/**
 * PriorityTaskList: Renders a priority-sorted list of tasks.
 *
 * This component is the main task display in the "Do" view.
 * It shows tasks ordered by their priority score with the most
 * important/urgent tasks at the top.
 *
 * Displays an empty state message when there are no tasks.
 *
 * @example
 * <PriorityTaskList
 *   tasks={priorityTasks}
 *   onToggle={(id) => toggleCompletion(id)}
 *   onTitleTap={(id) => openEditor(id)}
 * />
 */
export function PriorityTaskList({
  tasks,
  onToggle,
  onTitleTap,
}: PriorityTaskListProps) {
  if (tasks.length === 0) {
    return (
      <Text c="dimmed" py="xl" ta="center">
        No tasks needed. Great job!
      </Text>
    );
  }

  return (
    <Stack gap="xs">
      {tasks.map((task) => (
        <TaskRow
          key={task.id}
          onToggle={onToggle}
          task={task}
          {...(onTitleTap && {onTitleTap})}
        />
      ))}
    </Stack>
  );
}
