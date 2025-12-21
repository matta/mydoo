import {Box, Button, Stack} from '@mantine/core';
import {type TaskID, type TunnelNode} from '@mydoo/tasklens';
/**
 * TodoList: Renders a list of tasks at a given level in the hierarchy.
 *
 * This component handles:
 * - Rendering each task item with TodoItem component.
 * - Recursively rendering child task lists for expanded items.
 * - Providing an "Add item" interface at this level.
 *
 * The component enforces a maximum visible depth of 3 levels (0, 1, 2) to
 * prevent excessive nesting in the UI.
 *
 * @remarks
 * This is a recursive component - it renders TodoList for each expanded
 * child, which allows the task tree to be displayed to arbitrary depth.
 */
import {useState} from 'react';

import {InlineInput} from './InlineInput';
import {TodoItem} from './TodoItem';

/**
 * Props for the TodoList component.
 *
 * @property list - Array of tasks to display at this level.
 * @property basePath - Navigation path to the parent of this list (array of task IDs).
 * @property depth - Current nesting depth (0 = root of the current view).
 * @property expandedIds - Set of task IDs that should show their children.
 * @property editingId - ID of the task currently being edited, or undefined.
 * @property onToggleDone - Callback when a task's completion is toggled.
 * @property onToggleExpand - Callback when a task's expansion is toggled.
 * @property onStartEdit - Callback when editing begins on a task.
 * @property onSaveEdit - Callback when an edit is saved.
 * @property onCancelEdit - Callback when editing is cancelled.
 * @property onAddItem - Callback when a new item is added at this level.
 */
interface TodoListProps {
  basePath: TaskID[];
  depth: number;
  editingId: TaskID | undefined;
  expandedIds: Set<TaskID>;
  list: TunnelNode[];
  onAddItem: (path: TaskID[], title: string) => void;
  onCancelEdit: () => void;
  onSaveEdit: (path: TaskID[], newTitle: string) => void;
  onStartEdit: (id: TaskID) => void;
  onToggleDone: (path: TaskID[]) => void;
  onToggleExpand: (fullPath: TaskID[]) => void;
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
  if (depth > 2) return undefined;

  const hasItems = list.length > 0;

  return (
    <Stack gap={4} pl={depth > 0 ? 'lg' : 0}>
      {!hasItems && (
        <Box c="dimmed" fs="italic" fz="sm" py="xs">
          No items
        </Box>
      )}

      {list.map(item => {
        const id = item.id;
        const fullPath = [...basePath, id];
        const isExpanded = expandedIds.has(id);
        const isEditing = editingId === id;

        return (
          <Box key={id}>
            <TodoItem
              id={id}
              isEditing={isEditing}
              isExpanded={isExpanded}
              item={item}
              onCancelEdit={onCancelEdit}
              onSaveEdit={onSaveEdit}
              onStartEdit={onStartEdit}
              onToggleDone={onToggleDone}
              onToggleExpand={onToggleExpand}
              path={fullPath}
            />

            {/* Render children if expanded */}
            {isExpanded && (
              <TodoList
                basePath={fullPath}
                depth={depth + 1}
                editingId={editingId}
                expandedIds={expandedIds}
                list={item.children}
                onAddItem={onAddItem}
                onCancelEdit={onCancelEdit}
                onSaveEdit={onSaveEdit}
                onStartEdit={onStartEdit}
                onToggleDone={onToggleDone}
                onToggleExpand={onToggleExpand}
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
            onCancel={() => {
              setIsAdding(false);
            }}
            onSave={title => {
              onAddItem(basePath, title);
              setIsAdding(false);
            }}
            placeholder="New task title..."
          />
        ) : (
          <Button
            color="gray"
            fullWidth
            justify="flex-start"
            leftSection="+"
            onClick={() => {
              setIsAdding(true);
            }}
            pl={0}
            size="sm"
            variant="subtle"
          >
            Add Item
          </Button>
        )}
      </Box>
    </Stack>
  );
}
