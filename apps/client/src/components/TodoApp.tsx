import { useState, useCallback } from "react";
import { useDocument, useRepo } from "@automerge/automerge-repo-react-hooks";
import type { AnyDocumentId } from "@automerge/automerge-repo";
import { Breadcrumbs } from "./Breadcrumbs";
import { TodoList } from "./TodoList";
import { getBreadcrumbs, getListAtPath, generateId } from "../lib/todoUtils";
import type { TodoDoc, TodoList as TodoListType } from "../lib/types";
import "../styles/TodoApp.css";

export function TodoApp() {
  const repo = useRepo();

  // Get or create document URL from hash
  const [docUrl] = useState(() => {
    const hash = window.location.hash.slice(1);
    if (hash) return hash as AnyDocumentId;

    // Create new document if none exists
    const handle = repo.create<TodoDoc>();
    handle.change((doc) => {
      doc.todos = {};
      doc.todoOrder = [];
    });
    const url = handle.url;
    window.location.hash = url;
    return url;
  });

  const [doc, changeDoc] = useDocument<TodoDoc>(docUrl);

  // UI State
  const [viewPath, setViewPath] = useState<string[]>([]);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [editingId, setEditingId] = useState<string | null>(null);

  // Handlers (Moved before early returns to satisfy Hook rules)
  const handleNavigate = useCallback((path: string[]) => {
    setViewPath(path);
  }, []);

  const handleToggleDone = useCallback(
    (fullPath: string[]) => {
      changeDoc((doc) => {
        let current: TodoListType = doc;
        for (let i = 0; i < fullPath.length - 1; i++) {
          const id = fullPath[i];
          if (!id) continue;
          const next = current.todos[id];
          if (!next) return;
          current = next.children;
        }
        const id = fullPath[fullPath.length - 1];
        if (!id) return;
        const item = current.todos[id];
        if (item) {
          item.done = !item.done;
        }
      });
    },
    [changeDoc],
  );

  const handleToggleExpand = useCallback(
    (fullPath: string[]) => {
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

  // Recursive cleanup helper
  const cleanupList = useCallback((list: TodoListType) => {
    function recursiveCleanup(currentList: TodoListType) {
      // 1. Clean children first (depth-first)
      for (const id of currentList.todoOrder) {
        const item = currentList.todos[id];
        if (item) {
          recursiveCleanup(item.children);
        }
      }

      // 2. Collect done item IDs
      const doneIds = new Set<string>();
      for (const [id, item] of Object.entries(currentList.todos)) {
        if (item.done) {
          doneIds.add(id);
        }
      }

      // 3. Delete done items
      for (const id of doneIds) {
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete currentList.todos[id];
      }

      // 4. Update order array
      const newOrder = currentList.todoOrder.filter((id) => !doneIds.has(id));
      currentList.todoOrder.splice(
        0,
        currentList.todoOrder.length,
        ...newOrder,
      );
    }

    recursiveCleanup(list);
  }, []);

  const handleCleanup = useCallback(() => {
    changeDoc((doc) => {
      cleanupList(doc);
    });
  }, [changeDoc, cleanupList]);

  const handleStartEdit = useCallback((id: string) => {
    setEditingId(id);
  }, []);

  const handleSaveEdit = useCallback(
    (fullPath: string[], newTitle: string) => {
      changeDoc((doc) => {
        let current: TodoListType = doc;
        for (let i = 0; i < fullPath.length - 1; i++) {
          const pathId = fullPath[i];
          if (!pathId) continue;
          const next = current.todos[pathId];
          if (!next) return;
          current = next.children;
        }
        const id = fullPath[fullPath.length - 1];
        if (!id) return;
        const item = current.todos[id];
        if (item) {
          item.title = newTitle;
        }
      });
      setEditingId(null);
    },
    [changeDoc],
  );

  const handleCancelEdit = useCallback(() => {
    setEditingId(null);
  }, []);

  const handleAddItem = useCallback(
    (basePath: string[], title: string) => {
      changeDoc((doc) => {
        let current: TodoListType = doc;
        for (const id of basePath) {
          const next = current.todos[id];
          if (!next) return;
          current = next.children;
        }

        const newId = generateId();
        current.todos[newId] = {
          title,
          done: false,
          children: { todos: {}, todoOrder: [] },
        };
        // Add to beginning of list (newest first)
        current.todoOrder.unshift(newId);
      });
    },
    [changeDoc],
  );

  // Early returns
  if (!doc) {
    return <div className="loading">Loading...</div>;
  }

  // Get the list at the current view path
  const currentList = getListAtPath(doc, viewPath);
  if (!currentList) {
    // Invalid path, reset to root
    setViewPath([]);
    return <div className="loading">Resetting view...</div>;
  }

  const breadcrumbs = getBreadcrumbs(doc, viewPath);

  return (
    <div className="todo-app">
      <header className="app-header">
        <h1>Mydoo</h1>
        <button className="cleanup-btn" onClick={handleCleanup}>
          🧹 Cleanup Done
        </button>
      </header>

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
    </div>
  );
}
