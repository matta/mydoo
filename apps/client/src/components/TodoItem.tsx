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
  const hasChildren = item.children.todoOrder.length > 0;
  const canComplete = canMarkDone(item);

  // Determine expand button display
  let expandIcon: string;
  let expandDisabled: boolean;
  if (hasChildren) {
    expandIcon = isExpanded ? "▼" : "▶";
    expandDisabled = false;
  } else {
    expandIcon = "○";
    expandDisabled = true;
  }

  return (
    <div className={`todo-item ${item.done ? "done" : ""}`}>
      {/* Expand/Collapse Button */}
      <button
        className="expand-btn"
        onClick={() => {
          onToggleExpand(path);
        }}
        disabled={expandDisabled}
        aria-label={
          hasChildren ? (isExpanded ? "Collapse" : "Expand") : "No children"
        }
      >
        {expandIcon}
      </button>

      {/* Checkbox */}
      <input
        type="checkbox"
        checked={item.done}
        disabled={!canComplete}
        onChange={() => {
          onToggleDone(path);
        }}
        title={canComplete ? "" : "Complete all children first"}
      />

      {/* Title (editable or display) */}
      {isEditing ? (
        <InlineInput
          initialValue={item.title}
          onSave={(newTitle) => {
            onSaveEdit(path, newTitle);
          }}
          onCancel={() => {
            onCancelEdit();
          }}
        />
      ) : (
        <span
          className="todo-title"
          onClick={() => {
            onStartEdit(id);
          }}
        >
          {item.title}
        </span>
      )}
    </div>
  );
}
