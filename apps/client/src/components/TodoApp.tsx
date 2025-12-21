/**
 * TodoApp: Main application component for the task management UI.
 *
 * This component serves as the root of the task management interface. It:
 * 1. Connects to an Automerge document (creating one if needed).
 * 2. Manages UI state for navigation, expansion, and editing.
 * 3. Renders the task list with breadcrumb navigation.
 *
 * The document URL is stored in the browser's URL hash, allowing users to
 * share or bookmark specific task lists.
 *
 * @remarks
 * This component uses several React patterns that may be unfamiliar:
 * - **useState with initializer**: `useState(() => { ... })` runs the function
 *   once on mount to compute the initial value.
 * - **useCallback**: Memoizes functions to prevent unnecessary re-renders when
 *   passed as props to child components.
 * - **useTunnel hook**: Provides reactive access to the Automerge document.
 */
import { useState, useCallback } from "react";
import { useRepo } from "@automerge/automerge-repo-react-hooks";
import type { AnyDocumentId } from "@automerge/automerge-repo";
import { Container, Title, Button, Group, Loader, Stack } from "@mantine/core";
import { Breadcrumbs } from "./Breadcrumbs";
import { TodoList } from "./TodoList";
import { getBreadcrumbs, getListAtPath } from "../lib/todoUtils";
import {
  useTunnel,
  type TunnelNode,
  type TunnelState,
  type TaskID,
  TaskStatus,
} from "@mydoo/tasklens";

export function TodoApp() {
  const repo = useRepo();

  // Get or create document URL from hash
  const [docUrl] = useState(() => {
    const hash = window.location.hash.slice(1);
    if (hash) return hash as AnyDocumentId;

    // Create new document if none exists
    // We create a fresh tunnel state
    const handle = repo.create<TunnelState>();
    handle.change((doc) => {
      // Init logic matching TunnelStore default but explicit here since we use raw repo create
      doc.tasks = {};
      doc.places = {};
      doc.rootTaskIds = [];
      doc.nextTaskId = 1;
      doc.nextPlaceId = 1;
    });
    const url = handle.url;
    window.location.hash = url;
    return url;
  });

  const { tasks, ops } = useTunnel(docUrl);

  // UI State
  const [viewPath, setViewPath] = useState<TaskID[]>([]);
  const [expandedIds, setExpandedIds] = useState<Set<TaskID>>(new Set());
  const [editingId, setEditingId] = useState<TaskID | null>(null);

  // Handlers
  const handleNavigate = useCallback((path: TaskID[]) => {
    setViewPath(path);
  }, []);

  const handleToggleDone = useCallback(
    (fullPath: TaskID[]) => {
      const id = fullPath[fullPath.length - 1];
      if (id) ops.toggleDone(id);
    },
    [ops],
  );

  const handleToggleExpand = useCallback(
    (fullPath: TaskID[]) => {
      const id = fullPath[fullPath.length - 1];
      if (id === undefined) return;

      setExpandedIds((prev) => {
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
              setViewPath((prev) => [...prev, nextRootId]);
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
          // Note: deleting parent implicitly orphans children.
          // When parent is done, we skip traversing its children.
        } else {
          traverse(node.children);
        }
      }
    };
    traverse(tasks);

    idsToDelete.forEach((id) => {
      ops.delete(id);
    });
  }, [tasks, ops]);

  const handleStartEdit = useCallback((id: TaskID) => {
    setEditingId(id);
  }, []);

  const handleSaveEdit = useCallback(
    (fullPath: TaskID[], newTitle: string) => {
      const id = fullPath[fullPath.length - 1];
      if (id) {
        ops.update(id, { title: newTitle });
      }
      setEditingId(null);
    },
    [ops],
  );

  const handleCancelEdit = useCallback(() => {
    setEditingId(null);
  }, []);

  const handleAddItem = useCallback(
    (basePath: TaskID[], title: string) => {
      // basePath is array of IDs. The last ID is the parent.
      // If basePath is empty, parent is null.
      const parentId =
        basePath.length > 0 ? basePath[basePath.length - 1] : null;
      ops.add({ title, parentId: parentId ?? null });
    },
    [ops],
  );

  // Early returns

  // Get the list at the current view path
  // `tasks` is the root list (TunnelNode[])
  // getListAtPath navigates down
  const currentList = getListAtPath(tasks, viewPath);

  if (!currentList) {
    // Invalid path, reset to root
    setViewPath([]);
    return (
      <Container size="sm" py="xl">
        <Loader />
        {/* Resetting view... */}
      </Container>
    );
  }

  const breadcrumbs = getBreadcrumbs(tasks, viewPath);

  return (
    <Container size="sm" py="xl">
      <Group justify="space-between" mb="lg">
        <Title order={1}>Mydoo</Title>
        <Button
          color="red"
          variant="subtle"
          size="sm"
          onClick={handleCleanup}
          leftSection="🧹"
        >
          Cleanup Done
        </Button>
      </Group>

      <Stack gap="md">
        <Breadcrumbs crumbs={breadcrumbs} onNavigate={handleNavigate} />

        <TodoList
          list={currentList}
          basePath={viewPath}
          depth={0}
          expandedIds={expandedIds}
          editingId={editingId}
          onToggleDone={handleToggleDone}
          onToggleExpand={handleToggleExpand}
          onStartEdit={handleStartEdit}
          onSaveEdit={handleSaveEdit}
          onCancelEdit={handleCancelEdit}
          onAddItem={handleAddItem}
        />
      </Stack>
    </Container>
  );
}
