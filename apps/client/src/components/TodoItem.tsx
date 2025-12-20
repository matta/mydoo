import { ActionIcon, Checkbox, Group, Text } from "@mantine/core";
import { InlineInput } from "./InlineInput";
import type { TodoItem as TodoItemType } from "../lib/types";
import { canMarkDone } from "../lib/todoUtils";

interface TodoItemProps {
  id: string;
  item: TodoItemType;
  path: string[]; // Full path to this item (including its own id)
  isExpanded: boolean;
  isEditing: boolean;
  onToggleDone: (path: string[]) => void;
  onToggleExpand: (path: string[]) => void;
  onStartEdit: (id: string) => void;
  onSaveEdit: (path: string[], newTitle: string) => void;
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
      className={`todo-item ${item.done ? "done" : ""}`}
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
        checked={item.done}
        disabled={!canComplete}
        onChange={() => {
          onToggleDone(path);
        }}
        title={canComplete ? "" : "Complete all children first"}
        size="sm"
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
              textDecoration: item.done ? "line-through" : "none",
              color: item.done ? "var(--mantine-color-dimmed)" : "inherit",
            }}
          >
            {item.title}
          </Text>
        )}
      </Group>
    </Group>
  );
}
