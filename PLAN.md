# Mydoo Svelte → React Conversion Plan

This document is an **in-order task list** for converting the Svelte client (`apps/svelte-client`) to React (`apps/client`). Complete tasks sequentially, checking off each item as you go.

---

## Reference Materials

Before starting, review these sections for context. Do not modify or skip them.

<details>
<summary><strong>Architecture Overview</strong></summary>

The Svelte application is a **local-first** hierarchical todo list (outliner) with:

- **Real-time sync** via Automerge CRDT documents
- **Offline-first** via IndexedDB storage
- **Collaborative editing** via WebSocket connection to `wss://sync.automerge.org`
- **Nested todo structure** supporting infinite depth with 3-level visible display
- **SvelteKit with SSR disabled** (client-side only due to Automerge WASM requirements)

| Decision                         | Rationale                                                                                           |
| -------------------------------- | --------------------------------------------------------------------------------------------------- |
| SSR disabled (`ssr = false`)     | Automerge uses WebAssembly and browser APIs (IndexedDB) that don't work server-side                 |
| Dynamic imports in `onMount`     | Prevents mobile browser crashes from static Automerge imports (WASM issues on iPhone HTTP contexts) |
| Async singleton pattern for Repo | Enables lazy loading while ensuring single Repo instance                                            |

</details>

<details>
<summary><strong>Data Structures</strong></summary>

```typescript
interface TodoItem {
  title: string;
  done: boolean;
  children: TodoList;
}

interface TodoList {
  todos: { [id: string]: TodoItem };
  todoOrder: string[]; // Maintains insertion order (newest first)
}

// Root document is a TodoList
interface TodoDoc extends TodoList {}
```

</details>

<details>
<summary><strong>File Structure Mapping</strong></summary>

| Svelte File                     | React Equivalent                                      |
| ------------------------------- | ----------------------------------------------------- |
| `src/lib/db.ts`                 | `src/lib/db.ts` (verbatim copy, change browser check) |
| `src/routes/+layout.ts`         | N/A (React is client-only by default)                 |
| `src/routes/+layout.svelte`     | `src/App.tsx` (wrap with RepoProvider)                |
| `src/routes/+page.svelte`       | `src/components/TodoApp.tsx` + children               |
| CSS in `+page.svelte` `<style>` | `src/styles/TodoApp.css`                              |
| `src/lib/assets/favicon.svg`    | `public/favicon.svg`                                  |

</details>

---

## Task List

### Phase 1: Project Setup

#### Task 1.1: Install Automerge Dependencies

**File:** `apps/client/package.json`

**Steps:**

1. Open a terminal in the `apps/client` directory.
2. Run the following command:
   ```bash
   pnpm add @automerge/automerge-repo @automerge/automerge-repo-network-websocket @automerge/automerge-repo-storage-indexeddb @automerge/automerge-repo-react-hooks
   ```
3. Verify `package.json` now contains these in `dependencies`:
   ```json
   {
     "@automerge/automerge-repo": "^2.5.1",
     "@automerge/automerge-repo-network-websocket": "^2.5.1",
     "@automerge/automerge-repo-storage-indexeddb": "^2.5.1",
     "@automerge/automerge-repo-react-hooks": "^2.5.1"
   }
   ```

- [ ] **Checkpoint:** Dependencies installed and listed in `package.json`.

---

#### Task 1.2: Install Vite WASM Plugins

**File:** `apps/client/package.json`

**Steps:**

1. In the `apps/client` directory, run:
   ```bash
   pnpm add -D vite-plugin-wasm vite-plugin-top-level-await
   ```
2. Verify `package.json` now contains these in `devDependencies`:
   ```json
   {
     "vite-plugin-wasm": "^3.5.0",
     "vite-plugin-top-level-await": "^1.6.0"
   }
   ```

- [ ] **Checkpoint:** WASM plugins installed.

---

#### Task 1.3: Configure Vite for WASM

**File:** `apps/client/vite.config.ts`

**Steps:**

1. Open `apps/client/vite.config.ts`.
2. Add these imports at the top of the file:
   ```typescript
   import wasm from "vite-plugin-wasm";
   import topLevelAwait from "vite-plugin-top-level-await";
   ```
3. Add both plugins to the `plugins` array. The final config should look like:

   ```typescript
   import { defineConfig } from "vite";
   import react from "@vitejs/plugin-react";
   import wasm from "vite-plugin-wasm";
   import topLevelAwait from "vite-plugin-top-level-await";
   // ... other imports (PWA, etc.)

   export default defineConfig({
     plugins: [
       react(),
       wasm(),
       topLevelAwait(),
       // ... existing plugins like VitePWA
     ],
     // ... rest of config
   });
   ```

4. Save the file.

- [ ] **Checkpoint:** Run `pnpm dev` and verify no import errors related to WASM.

---

### Phase 2: Create Type Definitions and Library Code

#### Task 2.1: Create TypeScript Interfaces

**File:** `apps/client/src/lib/types.ts` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/lib/types.ts`.
2. Add the following content **exactly**:

   ```typescript
   export interface TodoItem {
     title: string;
     done: boolean;
     children: TodoList;
   }

   export interface TodoList {
     todos: { [id: string]: TodoItem };
     todoOrder: string[];
   }

   export interface TodoDoc extends TodoList {}
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with correct interfaces.

---

#### Task 2.2: Create Automerge Repo Singleton (Idiomatic Approach)

**File:** `apps/client/src/lib/db.ts` (NEW FILE)

> [!IMPORTANT]
> Try the **idiomatic approach** first. Only use the fallback if mobile crashes occur.

**Steps:**

1. Create a new file at `apps/client/src/lib/db.ts`.
2. Add the following content:

   ```typescript
   import { Repo } from "@automerge/automerge-repo";
   import { BrowserWebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";
   import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb";

   // Singleton repo instance
   export const repo = new Repo({
     network: [new BrowserWebSocketClientAdapter("wss://sync.automerge.org")],
     storage: new IndexedDBStorageAdapter(),
   });
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with repo export.

---

#### Task 2.2-FALLBACK: Create Automerge Repo Singleton (Dynamic Import)

> [!CAUTION]
> Only do this task if Task 2.2 causes white-screen crashes on mobile devices.

**File:** `apps/client/src/lib/db.ts` (REPLACE)

**Steps:**

1. Replace the contents of `apps/client/src/lib/db.ts` with:

   ```typescript
   let repoPromise: Promise<any> | null = null;
   let repoInstance: any = null;

   export async function getRepo() {
     if (typeof window === "undefined") return null;
     if (repoInstance) return repoInstance;
     if (repoPromise) return repoPromise;

     repoPromise = (async () => {
       try {
         console.log("[DB] Dynamically importing Automerge...");
         const [
           { Repo },
           { IndexedDBStorageAdapter },
           { BrowserWebSocketClientAdapter },
         ] = await Promise.all([
           import("@automerge/automerge-repo"),
           import("@automerge/automerge-repo-storage-indexeddb"),
           import("@automerge/automerge-repo-network-websocket"),
         ]);

         repoInstance = new Repo({
           network: [
             new BrowserWebSocketClientAdapter("wss://sync.automerge.org"),
           ],
           storage: new IndexedDBStorageAdapter(),
         });
         return repoInstance;
       } catch (e) {
         console.error("[DB] Failed to init:", e);
         throw e;
       }
     })();
     return repoPromise;
   }
   ```

2. Save the file.
3. You will also need to modify all components that use the repo to handle the async initialization.

- [ ] **Checkpoint (Fallback):** File uses dynamic imports and getRepo() function.

---

### Phase 3: Create React Context and Hooks

#### Task 3.1: Create Repo Provider (Idiomatic)

**File:** `apps/client/src/hooks/useRepo.tsx` (NEW FILE)

**Steps:**

1. Create a new directory: `apps/client/src/hooks/`
2. Create a new file at `apps/client/src/hooks/useRepo.tsx`.
3. Add the following content:

   ```tsx
   import { createContext, useContext } from "react";
   import { repo } from "../lib/db";

   const RepoContext = createContext(repo);

   export function RepoProvider({ children }: { children: React.ReactNode }) {
     return (
       <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>
     );
   }

   export function useRepo() {
     return useContext(RepoContext);
   }
   ```

4. Save the file.

- [ ] **Checkpoint:** File exists with RepoProvider and useRepo exports.

---

#### Task 3.2: Wrap App with RepoProvider

**File:** `apps/client/src/main.tsx`

**Steps:**

1. Open `apps/client/src/main.tsx`.
2. Import the RepoProvider:
   ```typescript
   import { RepoProvider } from "./hooks/useRepo";
   ```
3. Wrap the `<App />` component with `<RepoProvider>`:
   ```tsx
   ReactDOM.createRoot(document.getElementById("root")!).render(
     <React.StrictMode>
       <RepoProvider>
         <App />
       </RepoProvider>
     </React.StrictMode>,
   );
   ```
4. Save the file.

- [ ] **Checkpoint:** App renders without errors when running `pnpm dev`.

---

### Phase 4: Create UI Components

#### Task 4.1: Create Utility Functions

**File:** `apps/client/src/lib/todoUtils.ts` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/lib/todoUtils.ts`.
2. Add the following content:

   ```typescript
   import type { TodoDoc, TodoList, TodoItem } from "./types";

   /**
    * Navigate into a nested TodoList by following a path of IDs.
    * @param doc - The root document
    * @param path - Array of todo IDs representing the path
    * @returns The TodoList at that path, or null if path is invalid
    */
   export function getListAtPath(
     doc: TodoDoc,
     path: string[],
   ): TodoList | null {
     let current: TodoList = doc;
     for (const id of path) {
       if (!current.todos?.[id]) return null;
       current = current.todos[id].children;
     }
     return current;
   }

   /**
    * Check if a todo item can be marked as done.
    * A todo can only be marked done if ALL its children are already done.
    * @param item - The todo item to check
    * @returns true if the item can be marked done
    */
   export function canMarkDone(item: TodoItem): boolean {
     if (!item.children?.todoOrder?.length) return true;

     return item.children.todoOrder.every((childId) => {
       const child = item.children.todos[childId];
       return child?.done;
     });
   }

   /**
    * Generate breadcrumb data for navigation.
    * @param doc - The root document
    * @param viewPath - Current path being viewed
    * @returns Array of breadcrumb objects with id, title, and path
    */
   export function getBreadcrumbs(doc: TodoDoc, viewPath: string[]) {
     const crumbs = [{ id: "root", title: "Root", path: [] as string[] }];

     let currentPath: string[] = [];
     let currentList: TodoList = doc;

     for (const id of viewPath) {
       if (!currentList.todos?.[id]) break;
       const item = currentList.todos[id];
       currentPath = [...currentPath, id];
       crumbs.push({ id, title: item.title, path: currentPath });
       currentList = item.children;
     }
     return crumbs;
   }

   /**
    * Generate a unique ID for new todo items.
    * Uses crypto.randomUUID() for guaranteed uniqueness.
    */
   export function generateId(): string {
     return crypto.randomUUID();
   }
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with all four utility functions.

---

#### Task 4.2: Create Breadcrumbs Component

**File:** `apps/client/src/components/Breadcrumbs.tsx` (NEW FILE)

**Steps:**

1. Create a new directory: `apps/client/src/components/`
2. Create a new file at `apps/client/src/components/Breadcrumbs.tsx`.
3. Add the following content:

   ```tsx
   interface BreadcrumbItem {
     id: string;
     title: string;
     path: string[];
   }

   interface BreadcrumbsProps {
     crumbs: BreadcrumbItem[];
     onNavigate: (path: string[]) => void;
   }

   export function Breadcrumbs({ crumbs, onNavigate }: BreadcrumbsProps) {
     return (
       <nav className="breadcrumbs" aria-label="Breadcrumb navigation">
         {crumbs.map((crumb, index) => {
           const isLast = index === crumbs.length - 1;

           return (
             <span key={crumb.id}>
               {index > 0 && <span className="separator"> / </span>}
               {isLast ? (
                 <strong className="current">{crumb.title}</strong>
               ) : (
                 <button
                   className="crumb-link"
                   onClick={() => onNavigate(crumb.path)}
                 >
                   {crumb.title}
                 </button>
               )}
             </span>
           );
         })}
       </nav>
     );
   }
   ```

4. Save the file.

- [ ] **Checkpoint:** File exists with Breadcrumbs component.

---

#### Task 4.3: Create InlineInput Component

**File:** `apps/client/src/components/InlineInput.tsx` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/components/InlineInput.tsx`.
2. Add the following content:

   ```tsx
   import { useState, useRef, useEffect } from "react";

   interface InlineInputProps {
     initialValue: string;
     onSave: (value: string) => void;
     onCancel: () => void;
     placeholder?: string;
   }

   export function InlineInput({
     initialValue,
     onSave,
     onCancel,
     placeholder = "Enter text...",
   }: InlineInputProps) {
     const [value, setValue] = useState(initialValue);
     const inputRef = useRef<HTMLInputElement>(null);

     // Auto-focus when mounted
     useEffect(() => {
       inputRef.current?.focus();
       inputRef.current?.select();
     }, []);

     function handleKeyDown(e: React.KeyboardEvent) {
       if (e.key === "Enter") {
         if (value.trim()) {
           onSave(value.trim());
         } else {
           onCancel();
         }
       } else if (e.key === "Escape") {
         onCancel();
       }
     }

     function handleBlur() {
       if (value.trim()) {
         onSave(value.trim());
       } else {
         onCancel();
       }
     }

     return (
       <input
         ref={inputRef}
         type="text"
         className="inline-input"
         value={value}
         onChange={(e) => setValue(e.target.value)}
         onKeyDown={handleKeyDown}
         onBlur={handleBlur}
         placeholder={placeholder}
       />
     );
   }
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with InlineInput component.

---

#### Task 4.4: Create TodoItem Component

**File:** `apps/client/src/components/TodoItem.tsx` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/components/TodoItem.tsx`.
2. Add the following content:

   ```tsx
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
     onStartEdit: (id: string, title: string) => void;
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
     const hasChildren = item.children?.todoOrder?.length > 0;
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
           onClick={() => onToggleExpand(path)}
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
           onChange={() => onToggleDone(path)}
           title={canComplete ? "" : "Complete all children first"}
         />

         {/* Title (editable or display) */}
         {isEditing ? (
           <InlineInput
             initialValue={item.title}
             onSave={(newTitle) => onSaveEdit(path, newTitle)}
             onCancel={onCancelEdit}
           />
         ) : (
           <span
             className="todo-title"
             onClick={() => onStartEdit(id, item.title)}
           >
             {item.title}
           </span>
         )}
       </div>
     );
   }
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with TodoItem component.

---

#### Task 4.5: Create TodoList Component (Recursive)

**File:** `apps/client/src/components/TodoList.tsx` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/components/TodoList.tsx`.
2. Add the following content:

   ```tsx
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
       <div
         className="todo-list"
         style={{ marginLeft: depth > 0 ? "1.5rem" : 0 }}
       >
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
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with recursive TodoList component.

---

#### Task 4.6: Create Main TodoApp Component

**File:** `apps/client/src/components/TodoApp.tsx` (NEW FILE)

**Steps:**

1. Create a new file at `apps/client/src/components/TodoApp.tsx`.
2. Add the following content:

   ```tsx
   import { useState, useCallback } from "react";
   import { useDocument } from "@automerge/automerge-repo-react-hooks";
   import { Breadcrumbs } from "./Breadcrumbs";
   import { TodoList } from "./TodoList";
   import { getBreadcrumbs, getListAtPath, generateId } from "../lib/todoUtils";
   import type { TodoDoc, TodoList as TodoListType } from "../lib/types";
   import { useRepo } from "../hooks/useRepo";

   export function TodoApp() {
     const repo = useRepo();

     // Get or create document URL from hash
     const [docUrl] = useState(() => {
       const hash = window.location.hash.slice(1);
       if (hash) return hash;

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

     // Loading state
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

     // Handlers
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

     const handleCleanup = useCallback(() => {
       changeDoc((doc) => {
         cleanupList(doc);
       });
     }, [changeDoc]);

     // Recursive cleanup helper (defined inside component for closure access)
     function cleanupList(list: TodoListType) {
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
   ```

3. Save the file.

- [ ] **Checkpoint:** File exists with complete TodoApp component.

---

#### Task 4.7: Update App.tsx to Use TodoApp

**File:** `apps/client/src/App.tsx`

**Steps:**

1. Open `apps/client/src/App.tsx`.
2. Replace the entire contents with:

   ```tsx
   import { TodoApp } from "./components/TodoApp";
   import "./App.css";

   function App() {
     return <TodoApp />;
   }

   export default App;
   ```

3. Save the file.

- [ ] **Checkpoint:** App renders TodoApp component.

---

### Phase 5: Add Styles

#### Task 5.1: Create TodoApp Styles

**File:** `apps/client/src/styles/TodoApp.css` (NEW FILE)

**Steps:**

1. Create a new directory: `apps/client/src/styles/`
2. Create a new file at `apps/client/src/styles/TodoApp.css`.
3. Add styling that matches the Svelte app. Port CSS from `apps/svelte-client/src/routes/+page.svelte`.
4. Import the CSS in `TodoApp.tsx` by adding at the top:
   ```tsx
   import "../styles/TodoApp.css";
   ```

- [ ] **Checkpoint:** Styles are applied and UI looks correct.

---

### Phase 6: Testing and Verification

#### Task 6.1: Run Development Server

**Steps:**

1. Open terminal in `apps/client`.
2. Run `pnpm dev`.
3. Open browser to the displayed URL (usually `http://localhost:5173`).

- [ ] **Checkpoint:** App loads without console errors.

---

#### Task 6.2: Test Core Functionality

**Manual testing checklist:**

- [ ] App displays "Loading..." initially, then shows empty todo list
- [ ] Click "+ Add Item" shows inline input
- [ ] Type a task name and press Enter → task appears
- [ ] Click on task title → inline edit input appears
- [ ] Edit title and press Enter → title updates
- [ ] Press Escape while editing → edit cancels
- [ ] Click checkbox → task is marked done (strikethrough)
- [ ] Add a child task to an existing task
- [ ] Cannot mark parent done until all children are done
- [ ] Expand/collapse buttons work correctly
- [ ] Breadcrumbs update when zooming into nested tasks
- [ ] Click breadcrumb → navigates to that level
- [ ] "Cleanup Done" button removes all completed tasks
- [ ] URL hash contains document ID
- [ ] Refresh page → same document loads

---

#### Task 6.3: Test Mobile (if fallback needed)

**Steps:**

1. Access the app from a mobile device on the same network.
2. If app crashes (white screen), implement Task 2.2-FALLBACK.
3. Re-test all functionality after fallback implementation.

- [ ] **Checkpoint:** App works on mobile devices.

---

### Phase 7: Production Build

#### Task 7.1: Build and Verify

**Steps:**

1. Run `pnpm build` in `apps/client`.
2. Run `pnpm preview` to test production build.
3. Verify all functionality works in production mode.

- [ ] **Checkpoint:** Production build works correctly.

---

## Summary of Files to Create

| File                             | Type   | Depends On                                    |
| -------------------------------- | ------ | --------------------------------------------- |
| `src/lib/types.ts`               | NEW    | None                                          |
| `src/lib/db.ts`                  | NEW    | None                                          |
| `src/lib/todoUtils.ts`           | NEW    | `types.ts`                                    |
| `src/hooks/useRepo.tsx`          | NEW    | `db.ts`                                       |
| `src/components/Breadcrumbs.tsx` | NEW    | None                                          |
| `src/components/InlineInput.tsx` | NEW    | None                                          |
| `src/components/TodoItem.tsx`    | NEW    | `types.ts`, `todoUtils.ts`, `InlineInput.tsx` |
| `src/components/TodoList.tsx`    | NEW    | `TodoItem.tsx`, `InlineInput.tsx`, `types.ts` |
| `src/components/TodoApp.tsx`     | NEW    | All above                                     |
| `src/styles/TodoApp.css`         | NEW    | None                                          |
| `src/main.tsx`                   | MODIFY | `useRepo.tsx`                                 |
| `src/App.tsx`                    | MODIFY | `TodoApp.tsx`                                 |
| `vite.config.ts`                 | MODIFY | None                                          |

---

_Document created: 2025-12-19_
