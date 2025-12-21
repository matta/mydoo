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
import { ActionIcon, Checkbox, Group, Text } from "@mantine/core";
import { InlineInput } from "./InlineInput";
import { type TunnelNode, type TaskID, TaskStatus } from "@mydoo/tasklens";
import { canMarkDone } from "../lib/todoUtils";

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
  item: TunnelNode;
  path: TaskID[];
  isExpanded: boolean;
  isEditing: boolean;
  onToggleDone: (path: TaskID[]) => void;
  onToggleExpand: (path: TaskID[]) => void;
  onStartEdit: (id: TaskID) => void;
  onSaveEdit: (path: TaskID[], newTitle: string) => void;
  onCancelEdit: () => void;
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
      gap="xs"
      wrap="nowrap"
      className={`todo-item ${item.status === TaskStatus.Done ? "done" : ""}`}
    >
      {/* Expand/Collapse Button */}
      <ActionIcon
        variant="subtle"
        size="sm"
        color="gray"
        onClick={() => {
          onToggleExpand(path);
        }}
        aria-label={isExpanded ? "Collapse" : "Expand"}
      >
        {isExpanded ? "▼" : "▶"}
      </ActionIcon>

      {/* Checkbox */}
      <Checkbox
        checked={item.status === TaskStatus.Done}
        disabled={!canComplete}
        onChange={() => {
          onToggleDone(path);
        }}
        title={canComplete ? "" : "Complete all children first"}
        size="sm"
        // indeterminate={item.status === "Pending" && hasDoneChildren?} // Maybe later
      />

      {/* Title (editable or display) */}
      <Group style={{ flex: 1 }}>
        {isEditing ? (
          <InlineInput
            initialValue={item.title}
            onSave={(newTitle) => {
              onSaveEdit(path, newTitle);
            }}
            onCancel={onCancelEdit}
          />
        ) : (
          <Text
            onClick={() => {
              onStartEdit(id);
            }}
            style={{
              cursor: "pointer",
              textDecoration:
                item.status === TaskStatus.Done ? "line-through" : "none",
              color:
                item.status === TaskStatus.Done
                  ? "var(--mantine-color-dimmed)"
                  : "inherit",
            }}
          >
            {item.title}
          </Text>
        )}
      </Group>
    </Group>
  );
}
