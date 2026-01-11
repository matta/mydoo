# React Primitives and State Audit

**Date:** 2026-01-10 **Scope:** `apps/client` and `packages/tasklens`

## Overview

The application follows a **Local-First / Sync-to-Redux** architecture.

- **Source of Truth:** Automerge Document (managed by `automerge-repo`).
- **Read Path:** Automerge Doc -> Redux Store -> React Components (via Redux
  Selectors).
- **Write Path:** React Components -> `useTaskActions` (React Hook wrapper) ->
  Automerge Doc (via `handle.change`).

State management is distinctly divided between **Automerge-backed Domain State**
and **Local UI State**.

## 1. Domain State (Automerge & Tasks)

Usage of primitives here is minimal and focused on infrastructure (syncing) or
form buffering.

### Infrastructure & Sync

- **`packages/tasklens/src/react/task-lens-provider.tsx`**
  - **React Hook:** `useEffect`
  - **Purpose:** Subscribes to Automerge document changes
    (`handle.on('change')`) and dispatches the snapshot to Redux
    (`dispatch(syncDoc(...))`).
- **`apps/client/src/viewmodel/use-document.ts`**
  - **React Hooks:** `useState`, `useEffect`
  - **Purpose:** Bootstraps the Automerge Document.
  - **State:** `docUrl`: (Persistence ID). Manages _which_ document is currently
    loaded (by reading/writing local storage). **(React State)**

### Buffered State (Form Editing)

- **`apps/client/src/components/modals/task-editor-modal.tsx`**
  - **React Hooks:** `useState`, `useEffect`
  - **Purpose:** Buffers Automerge task data into local React state for editing.
  - **State:**
    - `title`, `importance`, `effort`, `notes` (Task Fields)
    - `dueDate`, `leadTimeScalar`, `leadTimeUnit` (Scheduling)
    - `frequency`, `interval`, `isSequential` (Logic)
  - **Behavior:** Copies data _from_ props (Automerge) to state on mount/open.
    Writes back to Automerge only on "Save". **(React State)**

### Mutations

- **`packages/tasklens/src/react/hooks/use-task-actions.ts`**
  - **React Hook:** `useCallback`
  - **Purpose:** Wraps Automerge `handle.change` calls. No state stored, only
    stable function references. **(Automerge via React Hooks)**

## 2. Local UI State (Navigation & Ephemeral)

Extensive usage of `useState` and Context to manage view mechanics that are
_not_ persisted to Automerge.

### Navigation Hub

- **`apps/client/src/viewmodel/ui/use-navigation-state.tsx`**
  - **React Hooks:** `useState`, `useContext`, `useMemo`
  - **Purpose:** "Redux-replacement" for purely local, non-persisted UI state.
    **(Pure React State)**
  - **State:**
    - `activeTab`: ('do' | 'plan' | 'balance')
    - `expandedIds`: `Set<TaskID>` (Tree expansion state)
    - `viewPath`: `TaskID[]` (Drill-down stack)
    - `modal`: `ModalState` (Which modal is open and its args)
    - `lastCreatedTaskId`: (Flash highlight target)

### Component-Level UI State

- **`apps/client/src/components/primitives/quick-add-input.tsx`**
  - `useState`: Text input value.
- **`apps/client/src/components/shell/connection-modal.tsx`**
  - `useState`: Text input value for Document ID.
- **`apps/client/src/components/pwa/reload-prompt.tsx`**
  - `useRef`: Tracks if notification has been shown.
- **`apps/client/src/components/primitives/task-outline-item.tsx`**
  - `useRef`: DOM reference for scrolling (`scrollIntoView`).

## 3. Read-Only Projections

The majority of "Viewmodel" hooks do _not_ use React state primitives. They use
**Memoization** and **Redux Selectors** to derive data.

- `apps/client/src/viewmodel/projections/use-task-details.ts`
- `apps/client/src/viewmodel/projections/use-valid-parent-targets.ts`
- `apps/client/src/viewmodel/projections/use-task-tree.ts`
- `apps/client/src/viewmodel/ui/use-breadcrumbs.ts`

**Pattern:** `useSelector(selectTaskEntities)` + `useMemo(() => compute(...))`.

## Summary Table

| Category     | File                       | Primitives              | Purpose                 |
| :----------- | :------------------------- | :---------------------- | :---------------------- |
| **Infra**    | `use-document.ts`          | `useState`, `useEffect` | Doc ID / Existence      |
| **Infra**    | `task-lens-provider.tsx`   | `useEffect`             | Sync Automerge -> Redux |
| **Buffer**   | `task-editor-modal.tsx`    | `useState` (Heavy)      | Task Form State         |
| **UI Store** | `use-navigation-state.tsx` | `useState` (Heavy)      | Tabs, Expansion, Modals |
| **Local UI** | `quick-add-input.tsx`      | `useState`              | Input Value             |
| **Local UI** | `connection-modal.tsx`     | `useState`              | Input Value             |

## 4. Gap Analysis: Migrating React Primitives to Redux

This section explores the feasibility and implications of moving **every**
instance of React state (`useState`, `useContext`, `useReducer`) to the Redux
store.

### Overview

Migrating local React state to Redux involves:

1.  **Defining Slices:** Creating new Redux slices for UI state (e.g.,
    `ui/navigation`, `ui/forms`).
2.  **Dispatching Actions:** replacing `setState` with `dispatch(action)`.
3.  **Selecting State:** replacing state variables with
    `useSelector(select...)`.
4.  **Middleware:** handling side effects (currently in `useEffect`) via Thunks
    or Sagas (if strict adherence to "no React primitives" is required).

### Impact Assessment

| Category        | Complexity    | Benefits                                                     | Drawbacks                                                                                                                  |
| :-------------- | :------------ | :----------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------- |
| **Navigation**  | Low           | Time-travel debugging, persistent view state across reloads. | Boilerplate for simple toggles.                                                                                            |
| **Form Buffer** | Medium        | State persistence (don't lose form on close), global access. | Significant boilerplate for every input field. High Redux churn on keystrokes.                                             |
| **Local UI**    | High (Effort) | "Total Store" visibility.                                    | Overkill for ephemeral state (e.g., input values, hover states). Performance cost of global dispatch for frequent updates. |

### Migration Samples

#### A. Navigation State (High Value Candidate)

Currently, `useNavigationState` holds drill-down paths and tab selection.

**Current (React Context):**

```tsx
// use-navigation-state.tsx
export function NavigationProvider() {
  const [activeTab, setActiveTab] = useState<"do" | "plan">("do");
  const [viewPath, setViewPath] = useState<TaskID[]>([]);

  const pushView = (id: TaskID) => setViewPath((prev) => [...prev, id]);

  return (
    <Context.Provider value={{ activeTab, viewPath, pushView }}>
      ...
    </Context.Provider>
  );
}
```

**Target (Redux Slice):**

```tsx
// store/slices/navigation-slice.ts
const navigationSlice = createSlice({
  name: "navigation",
  initialState: { activeTab: "do", viewPath: [] },
  reducers: {
    setActiveTab(state, action) {
      state.activeTab = action.payload;
    },
    pushView(state, action) {
      state.viewPath.push(action.payload);
    },
  },
});

// Component Usage
// viewmodel/ui/use-navigation-state.ts
export function useNavigationState() {
  const dispatch = useDispatch();
  const { activeTab, viewPath } = useSelector((state) => state.navigation);

  return {
    activeTab,
    viewPath,
    setActiveTab: (tab) => dispatch(navActions.setActiveTab(tab)),
    pushView: (id) => dispatch(navActions.pushView(id)),
  };
}
```

#### B. Form State (Medium Value / High Effort)

Currently, `TaskEditorModal` buffers edits locally. Moving this to Redux allows
"drafts" to persist even if the modal closes accidentally.

**Current (React Local State):**

```tsx
// components/modals/task-editor-modal.tsx
export function TaskEditorModal({ task, onSave }) {
  // Reset state when 'task' prop changes
  const [title, setTitle] = useState(task?.title ?? "");

  const handleSave = () => onSave({ title });

  return <TextInput value={title} onChange={(e) => setTitle(e.target.value)} />;
}
```

**Target (Redux "Drafts" Slice):**

```tsx
// store/slices/drafts-slice.ts
const draftsSlice = createSlice({
  name: "drafts",
  initialState: { taskEditor: { title: "", importance: 0.5 } },
  reducers: {
    setDraftTitle(state, action) {
      state.taskEditor.title = action.payload;
    },
    initializeDraft(state, action) {
      state.taskEditor = action.payload;
    },
  },
});

// Component Usage
export function TaskEditorModal() {
  const dispatch = useDispatch();
  const title = useSelector((state) => state.drafts.taskEditor.title);

  // Still need useEffect to sync "prop" to "store" on open?
  // Or purely drive from store?
  useEffect(() => {
    if (task) dispatch(initializeDraft(task));
  }, [task]);

  return (
    <TextInput
      value={title}
      onChange={(e) => dispatch(setDraftTitle(e.target.value))}
    />
  );
}
```

_Critique:_ This introduces "Keystroke in Redux" performance concerns.
Debouncing dispatch or using a local buffer _inside_ the Redux-connected
component is often preferred.

#### C. Ephemeral UI State (Low Value / Anti-Pattern?)

Moving widely ephemeral state (like simple input values or open/closed menus) to
Redux is generally considered an anti-pattern unless strict specific
requirements exist (e.g. replayability).

**Current (React):**

```tsx
// quick-add-input.tsx
export function QuickAddInput() {
  const [value, setValue] = useState("");
  return <input value={value} onChange={(e) => setValue(e.target.value)} />;
}
```

**Target (Redux):**

```tsx
// store/slices/ui-transient-slice.ts
// Requires unique IDs for every component instance to avoid collisions
const uiSlice = createSlice({
  name: "ui",
  reducers: {
    setInputValue(state, action: PayloadAction<{ id: string; value: string }>) {
      state.inputs[action.payload.id] = action.payload.value;
    },
  },
});
```

_Critique:_ Extremely high boilerplate. Requires managing unique IDs for every
input component.

### Conclusion

1.  **Recommended:** Migrate **Navigation State** (`useNavigationState`) to
    Redux. It is global, persistent, and low-frequency.
2.  **Optional:** Migrate **Form/Draft State** (`TaskEditorModal`) to Redux
    _only if_ draft persistence is a user requirement.

## 5. TaskLens React-Removal Plan

The user identified that `packages/tasklens` currently contains React primitives
and specific bindings (`@automerge/automerge-repo-react-hooks`). This section
details the plan to completely decouple `tasklens` from React, transforming it
into a Pure Redux/Automerge Logic Library.

### Goal

Remove `packages/tasklens/src/react` entirely. `tasklens` should not depend on
`react`, `react-dom`, or `react-redux`.

### Architecture Shift

**Current:** `Client` -(uses)-> `TaskLensProvider` (React Component) -> `Hooks`
-> `Automerge Repo`

**Target:** `Client` -(uses)-> `TaskLens Middleware` (Redux) -> `Automerge Repo`

### Migration Steps

#### 1. Replace `TaskLensProvider` with Redux Middleware

Currently, `TaskLensProvider` syncs the Automerge doc to the Redux store via
`useEffect`. We will move this logic to a **Redux Middleware** or **Store
Enhancer**.

**New Artifact: `packages/tasklens/src/redux/sync-middleware.ts`**

```typescript
import { Repo, DocHandle } from "@automerge/automerge-repo";
import { Middleware } from "redux";
import { syncDoc } from "../store/slices/tasks-slice";

export function createTaskLensMiddleware(repo: Repo, docUrl: string): Middleware {
  return (store) => {
    const handle = repo.find(docUrl);

    // Initial Sync
    handle.doc().then(doc => {
      // ... runs validation/reconciliation logic ...
      store.dispatch(syncDoc({ proxyDoc: doc, parsedDoc: ... }));
    });

    // Listener
    handle.on("change", ({ doc }) => {
      store.dispatch(syncDoc({ proxyDoc: doc, parsedDoc: ... }));
    });

    return (next) => (action) => next(action);
  };
}
```

#### 2. Replace `useTaskActions` with Redux Thunks

Currently, `useTaskActions` wraps `handle.change` in React `useCallback`. We
will replace these with **Redux Thunks** that access the `DocHandle` via the
thunk's `extraArgument` or a singleton.

**New Artifact: `packages/tasklens/src/redux/thunks.ts`**

```typescript
import { createAsyncThunk } from "@reduxjs/toolkit";
import { TaskCreateInput } from "../types/ui";
import { createTaskOp } from "../persistence/ops";

// Assuming we inject { handle } into the thunk middleware
export const createTask = createAsyncThunk(
  "tasks/create",
  async (input: TaskCreateInput, { extra }) => {
    const { handle } = extra as { handle: DocHandle<TunnelState> };
    const id = crypto.randomUUID();

    handle.change((doc) => {
      createTaskOp(doc, { ...input, id });
    });

    return id;
  },
);
```

#### 3. Client-Side Integration

The client application will effectively "bind" the purely logic-based `tasklens`
library to React using standard Redux patterns.

**`apps/client/src/store.ts`**

```typescript
// Client constructs the Repo and Store
const repo = new Repo(...);
const docUrl = localStorage.getItem("docId");

const store = configureStore({
  reducer: rootReducer,
  middleware: (getDefault) =>
    getDefault().concat(createTaskLensMiddleware(repo, docUrl))
});
```

### Impact

- **Decoupling:** `tasklens` becomes framework-agnostic (could be reused in
  Svelte, Vue, or CLI tools).
- **Testability:** Logic can be tested purely via Redux actions without
  rendering React components.
- **Complexity:** Shifts complexity from "React Context" to "Redux Middleware
  configuration".
