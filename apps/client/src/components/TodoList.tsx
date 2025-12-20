import { useState } from "react";
import { Stack, Button, Box } from "@mantine/core";
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
  onStartEdit: (id: string) => void;
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
  // Don't render beyond depth 2 (3 levels visible: 0, 1, 2)
  if (depth > 2) return null;

  const hasItems = list.todoOrder.length > 0;

  return (
    <Stack gap={4} pl={depth > 0 ? "lg" : 0}>
      {!hasItems && (
        <Box c="dimmed" fs="italic" fz="sm" py="xs">
          No items
        </Box>
      )}

      {list.todoOrder.map((id) => {
        const item = list.todos[id];
        if (!item) return null;

        const fullPath = [...basePath, id];
        const isExpanded = expandedIds.has(id);
        const isEditing = editingId === id;

        return (
          <Box key={id}>
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
            {/* item.children is always defined in type, so we don't need to check existence */}
            {isExpanded && (
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
          </Box>
        );
      })}

      {/* Add Item UI */}
      <Box mt="xs">
        {isAdding ? (
          <InlineInput
            initialValue=""
            onSave={(title) => {
              onAddItem(basePath, title);
              setIsAdding(false);
            }}
            onCancel={() => {
              setIsAdding(false);
            }}
            placeholder="New task title..."
          />
        ) : (
          <Button
            variant="subtle"
            size="sm"
            color="gray"
            onClick={() => {
              setIsAdding(true);
            }}
            fullWidth
            justify="flex-start"
            pl={0}
            leftSection="+"
          >
            Add Item
          </Button>
        )}
      </Box>
    </Stack>
  );
}
