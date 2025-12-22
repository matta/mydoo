import {Button, Container, Group, Loader, Stack, Title} from '@mantine/core';
import {type TaskID, TaskStatus, type TunnelNode} from '@mydoo/tasklens';
import {useCallback, useState} from 'react';

import {
  useBreadcrumbs,
  useDocument,
  useTaskActions,
  useTaskTree,
  useTodoList,
} from '../viewmodel';
import {Breadcrumbs} from './Breadcrumbs';
import {TodoList} from './TodoList';

export function TodoApp() {
  const docUrl = useDocument();

  // --- ViewModel Integration ---
  const {tasks} = useTaskTree(docUrl);
  const actions = useTaskActions(docUrl);

  // UI State
  const [viewPath, setViewPath] = useState<TaskID[]>([]);
  const [expandedIds, setExpandedIds] = useState<Set<TaskID>>(new Set());
  const [editingId, setEditingId] = useState<TaskID | undefined>(undefined);

  // Projections
  const {currentList, isPathValid} = useTodoList(tasks, viewPath);
  const breadcrumbs = useBreadcrumbs(tasks, viewPath);

  // --- Handlers ---

  const handleNavigate = useCallback((path: TaskID[]) => {
    setViewPath(path);
  }, []);

  const handleToggleDone = useCallback(
    (fullPath: TaskID[]) => {
      const id = fullPath[fullPath.length - 1];
      if (id) actions.toggleDone(id);
    },
    [actions],
  );

  const handleToggleExpand = useCallback(
    (fullPath: TaskID[]) => {
      const id = fullPath[fullPath.length - 1];
      if (id === undefined) return;

      setExpandedIds(prev => {
        const next = new Set(prev);
        if (next.has(id)) {
          next.delete(id);
        } else {
          next.add(id);

          // Auto-zoom logic: if expanding at depth >= 2, shift view
          const depth = fullPath.length - viewPath.length - 1;
          if (depth >= 2) {
            const nextRootId = fullPath[viewPath.length];
            if (nextRootId !== undefined) {
              setViewPath(prev => [...prev, nextRootId]);
            }
          }
        }
        return next;
      });
    },
    [viewPath],
  );

  const handleCleanup = useCallback(() => {
    // Recursive cleanup of done items.
    // Collect IDs to delete first to avoid modifying while traversing.
    const idsToDelete: TaskID[] = [];
    const traverse = (nodes: TunnelNode[]) => {
      for (const node of nodes) {
        if (node.status === TaskStatus.Done) {
          idsToDelete.push(node.id);
        } else {
          traverse(node.children);
        }
      }
    };
    traverse(tasks);

    idsToDelete.forEach(id => {
      actions.deleteTask(id);
    });
  }, [tasks, actions]);

  const handleStartEdit = useCallback((id: TaskID) => {
    setEditingId(id);
  }, []);

  const handleSaveEdit = useCallback(
    (fullPath: TaskID[], newTitle: string) => {
      const id = fullPath[fullPath.length - 1];
      if (id) {
        actions.updateTask(id, {title: newTitle});
      }
      setEditingId(undefined);
    },
    [actions],
  );

  const handleCancelEdit = useCallback(() => {
    setEditingId(undefined);
  }, []);

  const handleAddItem = useCallback(
    (basePath: TaskID[], title: string) => {
      const parentId =
        basePath.length > 0 ? basePath[basePath.length - 1] : undefined;
      actions.addTask(title, parentId);
    },
    [actions],
  );

  // Check if path is valid, else reset
  if (!isPathValid) {
    if (viewPath.length > 0) {
      setViewPath([]); // Reset to root
    }
    return (
      <Container py="xl" size="sm">
        <Loader />
      </Container>
    );
  }

  if (!currentList) return undefined; // Should be handled by isPathValid check above

  return (
    <Container py="xl" size="sm">
      <Group justify="space-between" mb="lg">
        <Title order={1}>Mydoo</Title>
        <Button
          color="red"
          leftSection="🧹"
          onClick={handleCleanup}
          size="sm"
          variant="subtle"
        >
          Cleanup Done
        </Button>
      </Group>

      <Stack gap="md">
        <Breadcrumbs crumbs={breadcrumbs} onNavigate={handleNavigate} />

        <TodoList
          basePath={viewPath}
          depth={0}
          editingId={editingId}
          expandedIds={expandedIds}
          list={currentList}
          onAddItem={handleAddItem}
          onCancelEdit={handleCancelEdit}
          onSaveEdit={handleSaveEdit}
          onStartEdit={handleStartEdit}
          onToggleDone={handleToggleDone}
          onToggleExpand={handleToggleExpand}
        />
      </Stack>
    </Container>
  );
}
