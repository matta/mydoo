/**
 * TodoItem: A single task row in the task list.
 *
 * This component renders one task with:
 * - An expand/collapse button for tasks with children.
 * - A checkbox to toggle completion status.
 * - The task title (editable when in edit mode).
 *
 * The component is a "controlled" component, meaning all state and callbacks
 * are passed in as props from the parent. It does not manage its own state.
 */
import {ActionIcon, Checkbox, Group, Text} from '@mantine/core';
import {type TaskID, TaskStatus, type TunnelNode} from '@mydoo/tasklens';

import {canMarkDone} from '../lib/todo-utils';
import {InlineInput} from './inline-input';

/**
 * Props for the TodoItem component.
 *
 * @property id - The unique identifier of this task.
 * @property item - The task data including title, status, and children.
 * @property path - Full navigation path to this item (array of ancestor IDs + this ID).
 * @property isExpanded - Whether to show this item's children.
 * @property isEditing - Whether the title is currently being edited.
 * @property onToggleDone - Callback when the checkbox is clicked.
 * @property onToggleExpand - Callback when the expand/collapse button is clicked.
 * @property onStartEdit - Callback when the user initiates editing.
 * @property onSaveEdit - Callback when the user saves an edit.
 * @property onCancelEdit - Callback when the user cancels editing.
 */
interface TodoItemProps {
  id: TaskID;
  isEditing: boolean;
  isExpanded: boolean;
  item: TunnelNode;
  onCancelEdit: () => void;
  onSaveEdit: (path: TaskID[], newTitle: string) => void;
  onStartEdit: (id: TaskID) => void;
  onToggleDone: (path: TaskID[]) => void;
  onToggleExpand: (path: TaskID[]) => void;
  path: TaskID[];
}

export function TodoItem({
  id,
  item,
  path,
  isExpanded,
  isEditing,
  onToggleDone,
  onToggleExpand,
  onStartEdit,
  onSaveEdit,
  onCancelEdit,
}: TodoItemProps) {
  const canComplete = canMarkDone(item);

  return (
    <Group
      align="center"
      className={`todo-item ${item.status === TaskStatus.Done ? 'done' : ''}`}
      gap="xs"
      wrap="nowrap"
    >
      {/* Expand/Collapse Button */}
      <ActionIcon
        aria-label={isExpanded ? 'Collapse' : 'Expand'}
        color="gray"
        onClick={() => {
          onToggleExpand(path);
        }}
        size="sm"
        variant="subtle"
      >
        {isExpanded ? '▼' : '▶'}
      </ActionIcon>

      {/* Checkbox */}
      <Checkbox
        checked={item.status === TaskStatus.Done}
        disabled={!canComplete}
        onChange={() => {
          onToggleDone(path);
        }}
        size="sm"
        title={canComplete ? '' : 'Complete all children first'}
        // indeterminate={item.status === "Pending" && hasDoneChildren?} // Maybe later
      />

      {/* Title (editable or display) */}
      <Group style={{flex: 1}}>
        {isEditing ? (
          <InlineInput
            initialValue={item.title}
            onCancel={onCancelEdit}
            onSave={newTitle => {
              onSaveEdit(path, newTitle);
            }}
          />
        ) : (
          <Text
            onClick={() => {
              onStartEdit(id);
            }}
            style={{
              cursor: 'pointer',
              textDecoration:
                item.status === TaskStatus.Done ? 'line-through' : 'none',
              color:
                item.status === TaskStatus.Done
                  ? 'var(--mantine-color-dimmed)'
                  : 'inherit',
            }}
          >
            {item.title}
          </Text>
        )}
      </Group>
    </Group>
  );
}
