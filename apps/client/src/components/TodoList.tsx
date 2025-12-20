import { useState } from "react";
import { TodoItem } from "./TodoItem";
import { InlineInput } from "./InlineInput";
import type { TodoList as TodoListType } from "../lib/types";

interface TodoListProps {
  list: TodoListType;
  basePath: string[]; // Path to this list's parent
  depth: number; // Current depth (0 = root of view)
  expandedIds: Set<string>;
  editingId: string | null;
  onToggleDone: (path: string[]) => void;
  onToggleExpand: (fullPath: string[]) => void;
  onStartEdit: (id: string, title: string) => void;
  onSaveEdit: (path: string[], newTitle: string) => void;
  onCancelEdit: () => void;
  onAddItem: (path: string[], title: string) => void;
}

export function TodoList({
  list,
  basePath,
  depth,
  expandedIds,
  editingId,
  onToggleDone,
  onToggleExpand,
  onStartEdit,
  onSaveEdit,
  onCancelEdit,
  onAddItem,
}: TodoListProps) {
  const [isAdding, setIsAdding] = useState(false);

  // Don't render beyond depth 2 (3 levels visible: 0, 1, 2)
  if (depth > 2) return null;

  // Guard against missing data
  if (!list?.todoOrder) {
    return <div className="empty-list">No items</div>;
  }

  return (
    <div className="todo-list" style={{ marginLeft: depth > 0 ? "1.5rem" : 0 }}>
      {list.todoOrder.map((id) => {
        const item = list.todos[id];
        if (!item) return null;

        const fullPath = [...basePath, id];
        const isExpanded = expandedIds.has(id);
        const isEditing = editingId === id;

        return (
          <div key={id} className="todo-item-wrapper">
            <TodoItem
              id={id}
              item={item}
              path={fullPath}
              isExpanded={isExpanded}
              isEditing={isEditing}
              onToggleDone={onToggleDone}
              onToggleExpand={onToggleExpand}
              onStartEdit={onStartEdit}
              onSaveEdit={onSaveEdit}
              onCancelEdit={onCancelEdit}
            />

            {/* Render children if expanded */}
            {isExpanded && item.children && (
              <TodoList
                list={item.children}
                basePath={fullPath}
                depth={depth + 1}
                expandedIds={expandedIds}
                editingId={editingId}
                onToggleDone={onToggleDone}
                onToggleExpand={onToggleExpand}
                onStartEdit={onStartEdit}
                onSaveEdit={onSaveEdit}
                onCancelEdit={onCancelEdit}
                onAddItem={onAddItem}
              />
            )}
          </div>
        );
      })}

      {/* Add Item UI */}
      {depth === 0 && (
        <div className="add-item-container">
          {isAdding ? (
            <InlineInput
              initialValue=""
              onSave={(title) => {
                onAddItem(basePath, title);
                setIsAdding(false);
              }}
              onCancel={() => setIsAdding(false)}
              placeholder="New task title..."
            />
          ) : (
            <button className="add-btn" onClick={() => setIsAdding(true)}>
              + Add Item
            </button>
          )}
        </div>
      )}
    </div>
  );
}
