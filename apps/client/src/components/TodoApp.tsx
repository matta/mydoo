import { useState, useCallback } from "react";
import { useDocument } from "@automerge/automerge-repo-react-hooks";
import type { AnyDocumentId } from "@automerge/automerge-repo";
import { Breadcrumbs } from "./Breadcrumbs";
import { TodoList } from "./TodoList";
import { getBreadcrumbs, getListAtPath, generateId } from "../lib/todoUtils";
import type { TodoDoc, TodoList as TodoListType } from "../lib/types";
import { useRepo } from "../hooks/useRepo";
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
          current = current.todos[fullPath[i]].children;
        }
        const id = fullPath[fullPath.length - 1];
        current.todos[id].done = !current.todos[id].done;
      });
    },
    [changeDoc],
  );

  const handleToggleExpand = useCallback(
    (fullPath: string[]) => {
      const id = fullPath[fullPath.length - 1];

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
            setViewPath((prev) => [...prev, nextRootId]);
          }
        }
        return next;
      });
    },
    [viewPath],
  );

  const handleStartEdit = useCallback((id: string, _title: string) => {
    setEditingId(id);
  }, []);

  const handleSaveEdit = useCallback(
    (fullPath: string[], newTitle: string) => {
      changeDoc((doc) => {
        let current: TodoListType = doc;
        for (let i = 0; i < fullPath.length - 1; i++) {
          current = current.todos[fullPath[i]].children;
        }
        const id = fullPath[fullPath.length - 1];
        current.todos[id].title = newTitle;
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
          current = current.todos[id].children;
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

  // Recursive cleanup helper (Moved up to fix declaration order)
  const cleanupList = useCallback((list: TodoListType) => {
    if (!list.todos || !list.todoOrder) return;

    // 1. Clean children first (depth-first)
    for (const id of list.todoOrder) {
      if (list.todos[id]) {
        cleanupList(list.todos[id].children);
      }
    }

    // 2. Collect done item IDs
    const doneIds = new Set<string>();
    for (const [id, item] of Object.entries(list.todos)) {
      if (item.done) doneIds.add(id);
    }

    // 3. Delete done items
    for (const id of doneIds) {
      delete list.todos[id];
    }

    // 4. Update order array
    const newOrder = list.todoOrder.filter((id) => !doneIds.has(id));
    list.todoOrder.splice(0, list.todoOrder.length, ...newOrder);
  }, []);

  const handleCleanup = useCallback(() => {
    changeDoc((doc) => {
      cleanupList(doc);
    });
  }, [changeDoc, cleanupList]);

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
