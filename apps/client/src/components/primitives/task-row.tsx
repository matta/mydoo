import {Checkbox, Group, Paper, Text} from '@mantine/core';
import type {Task, TaskID} from '@mydoo/tasklens';

/**
 * Props for the TaskRow component.
 */
export interface TaskRowProps {
  /**
   * Callback fired when the completion checkbox is toggled.
   * @param id - The ID of the task being toggled.
   */
  onToggle: (id: TaskID) => void;

  /**
   * Optional callback fired when the task title is tapped/clicked.
   * Used to open the TaskEditorModal for editing task details.
   * @param id - The ID of the task that was tapped.
   */
  onTitleTap?: (id: TaskID) => void;

  /**
   * Optional inline styles to apply to the root Paper element.
   * Useful for positioning in virtualized lists.
   */
  style?: React.CSSProperties;

  /**
   * The task data to display. Contains title, status, importance, etc.
   */
  task: Task;
}

/**
 * TaskRow: A single task row in the priority list.
 *
 * Displays a task with:
 * - A checkbox to toggle completion status
 * - A clickable title that opens the TaskEditorModal (if onTitleTap provided)
 * - The task's importance score
 *
 * @example
 * <TaskRow
 *   task={myTask}
 *   onToggle={(id) => completeTask(id)}
 *   onTitleTap={(id) => openEditor(id)}
 * />
 */
export function TaskRow({task, onToggle, onTitleTap, style}: TaskRowProps) {
  return (
    <Paper p="xs" shadow="xs" style={style} withBorder>
      <Group align="center" gap="sm" wrap="nowrap">
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
          onClick={() => onTitleTap?.(task.id)}
          style={{
            userSelect: 'none',
            cursor: onTitleTap ? 'pointer' : 'default',
            textDecoration: task.status === 'Done' ? 'line-through' : undefined,
          }}
        >
          {task.title}
        </Text>

        <Text c="dimmed" size="xs">
          {task.importance.toFixed(2)}
        </Text>
      </Group>
    </Paper>
  );
}
