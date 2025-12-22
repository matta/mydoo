Here is the updated **Product Requirements Document (v1.6)**.

I have updated the **Schema** (Section 2.2) and the **Healer Routine** (Section 3.4) to explicitly handle Automerge array conflicts by enforcing a "First-Occurrence-Wins" deduplication strategy for `rootTaskIds` and `childIds`.

---

# Product Requirements Document (v1.6)

## 1. Core Philosophy

A local-first, synchronization-agnostic task management system. It eliminates "list rot" by dynamically promoting tasks based on a "Life Balance" algorithm (Target vs. Actual effort) and "Autofocus" principles (surfacing neglected tasks). The device is the source of truth.

## 2. Technical Architecture

### 2.1 The Stack

- **Language:** TypeScript (Strict `noImplicitAny`, `strictNullChecks`).
- **Framework:** React + Vite (PWA).
- **State:** Automerge (`@automerge/automerge-repo`).
- **Persistence:** IndexedDB (`automerge-repo-storage-indexeddb`).
- **Network:** WebSocket (`automerge-repo-network-websocket`).

### 2.2 Data Schema

The Automerge document structure is specified in **[automerge-schema.md](./automerge-schema.md)**.

> **Key Principle**: Automerge serves exclusively as the storage and merge layer. The rest of the application consumes plain TypeScript objects—never Automerge proxies directly.

## 3. The Logic Specification

### 3.1 The Inbox Logic

- **Structure:** The Inbox is a special **Top-Level Item (TLI)** with a reserved ID (`ROOT_INBOX_ID`). It sits at the same level as Goals (Health, Career).
- **Balance Exclusion:** Unlike other TLIs, the Inbox is **excluded** from the "Pie Chart" balance calculation. Items inside it do not get a "Balance" boost or penalty.
- **Protection:** It cannot be deleted or renamed.

### 3.2 Default Values for New Tasks

When creating a new task, apply the following defaults:

| Field                 | Default Value                                                  |
| --------------------- | -------------------------------------------------------------- |
| `importance`          | `1.0` (essential)                                              |
| `effort`              | Inherit from parent; root tasks default to `3` (average)       |
| `leadTimeDays`        | `0.33` (~8 hours)                                              |
| `notes`               | `undefined` (optional)                                         |
| `dueDate`             | `undefined` (no deadline)                                      |
| `placeId`             | Inherit from parent; root tasks default to `PLACE_ANYWHERE_ID` |
| `lastReviewTimestamp` | `Date.now()`                                                   |
| `status`              | `'active'`                                                     |
| `childIds`            | `[]`                                                           |

### 3.3 ID Generation Strategy (Duplicate Prevention)

To prevent duplicates when two devices complete the same recurring task offline, we use **Deterministic IDs**:

1. **Standard Tasks:** Random UUID (v4).
2. **Recurring Tasks:** `repeatConfig.seriesId + ":" + repeatConfig.iterationIndex`.

- _Effect:_ If two devices generate the next instance of "Laundry" offline, Automerge will merge them into a single entry (Last-Write-Wins).

### 3.4 Repetition Logic

- **Trigger:** User clicks the "Update" (Refresh) button.
- **Action:**

1. Find all tasks marked `completed` that have a `repeatConfig`.
2. For each, generate the _Next Task_:

- **Routinely:** `Next.dueDate = CompletionDate + intervalDays`.
- **Calendar:** `Next.dueDate = Previous.dueDate + intervalDays` (anchored to original schedule, even if past-due. Future enhancement: option to skip forward to next future occurrence).
- **Common:** `Next.repeatConfig.iterationIndex = Previous.repeatConfig.iterationIndex + 1`.
- **ID:** Uses the Deterministic ID formula.

3. Insert _Next Task_ into `tasks` map.
4. Archive _Previous Task_ (keep in map for history, but visually hide).

### 3.5 Data Integrity: The "Healer" Routine

The Healer runs on **Application Load** and **Post-Sync** to correct data anomalies caused by complex merges.

**Part A: Array Sanitization (Duplicate Removal)**
Automerge merges concurrent array insertions by keeping both. This creates duplicates in ordered lists (e.g., `['A', 'B', 'B', 'C']`).

- **Target:** `RootDoc.rootTaskIds` and every `Task.childIds`.
- **Strategy:** "Keep First Occurrence".
- Iterate through the array. Maintain a set of `seenIds`.
- If an ID is encountered for the first time, add to set.
- If an ID is encountered again (it is in `seenIds`), **delete** it from the array.

- **Safety:** Since all replicas agree to keep the first occurrence, no ID is ever completely lost, ensuring the structure remains intact.

**Part B: Recurring Task Deduplication**

- **Target:** All `active` tasks where `repeatConfig.type === 'routinely'`.
- **Logic:**

1. Group active tasks by `seriesId`.
2. If a group has **>1 active task**:

- Identify the task with the **highest** `iterationIndex` (The Winner).
- Identify all others (The Losers).
- **Action:** Explicitly set `status = 'deleted'` for all Losers.

### 3.6 Deletion Logic (Cascade with Confirmation)

When a user deletes a task that has children:

1. **Warning:** UI MUST present a confirmation dialog showing the count of descendants that will also be deleted (e.g., "Delete 'Project X' and 12 sub-tasks?").
2. **On Confirm:** Set `status = 'deleted'` on the target task AND all descendants (children, grandchildren, etc.).
3. **On Cancel:** No changes.

**Note:** Deleted tasks remain in the `tasks` map for history/undo purposes but are excluded from all views.

### 3.7 The Scoring Algorithm (Computed View)

The prioritization logic is fully specified in external documents:

- **[ALGORITHM.md](./algorithm.md):** Core 7-pass scoring algorithm (Credit tracking, Feedback/"Thermostat", Weight normalization, Lead time urgency).
- **[STALENESS.md](./staleness.md):** Autofocus/neglect mechanics and `lastReviewTimestamp` update rules.
- **[ALGORITHM_TEST_SUITE.md](./test-suite.md):** Compliance test fixtures.

This logic runs client-side on the flattened task list derived from the Automerge document.

## 4. UX/UI Specification

### 4.1 Global Navigation Paradigm

**Mobile (PWA) - Bottom Tab Bar**

1. **Do:** The computed, prioritized list.
2. **Plan:** The hierarchical tree view.
3. **Balance:** The "Pie Chart" slider adjustments.
4. **Context:** Context/Place management.

**Desktop (PWA) - Split Pane**

- **Left Pane:** Plan (Tree).
- **Right Pane:** Do (Computed List).
- **Top Bar:** Navigation to "Balance" and "Context", plus Global Settings.

### 4.2 The "Do" View (Primary)

- **Visual Structure:** A flat list of tasks sorted **purely by computed priority score**. There are no section headers or temporal groupings (e.g., "Overdue", "Today", "Later"). Overdue items, items due today, and items due later are intermixed—their position in the list is determined solely by the algorithm's priority calculation.
- **Filter Logic:** Shows tasks where `Visibility = True` per [ALGORITHM.md](./algorithm.md) Pass 1.
- **Context Filter:** Header dropdown (e.g., "Home", "Work"). Place filtering logic is defined in ALGORITHM.md §3.2.
- **Task Row Elements:**
  - **Left:** Checkbox.
  - **Center:** Task Title + Small Metadata text (Parent Project Name, Due Date icon).
  - **Visual Cues:**
    - **Overdue:** Red text/accent.
    - **Neglected/Stale:** Per [STALENESS.md](./staleness.md), show visual indicator when `StalenessFactor > 1.5`.
    - **Inbox:** Distinct border to denote unfiled status.

- **Interactions:**
  - **Checkbox:** Toggles task status. Checking sets `status = 'Done'` and triggers credit attribution. Unchecking reverts to `status = 'Pending'`.
  - **"Update/Refresh" Button:** Runs Repetition Logic (generates next instances), triggers "Healer", and **acknowledges completed tasks** (removing them from the Do list).
  - **Floating Action Button (Mobile):** Adds a new task directly to the **Inbox**.

> [!IMPORTANT]
> **Completed Task Visibility Lifecycle**
>
> When a task is marked "Done", it remains visible in the Do list with a strikethrough until acknowledged. This allows the user to see what they accomplished and to undo accidental completions.
>
> **The acknowledgment occurs when the user presses the Refresh button**, which sets `isAcknowledged = true` on finished tasks. Tasks with `isAcknowledged` are hidden from the Do view but remain in the document for history and the Balance view's credit tracking.
>
> **Persisted State**: The `isAcknowledged` flag is stored in the Automerge document (synced across devices).

### 4.3 The "Plan" View (Outline)

- **Structure:** Indented tree view. Infinite nesting supported.
- **Root Nodes:** Fixed "Inbox" node (pinned at top) + User TLIs.
- **Navigation (Hybrid)**:
  - **Desktop:** Tree with expand/collapse chevrons. Full outline visible.
  - **Mobile:** Drill-down navigation. Tapping a parent "zooms in" to show only its children. A **breadcrumb trail** at the top shows the current path and allows navigation back up.
- **Interaction:**
  - **Expand/Collapse (Desktop):** Chevrons (`>`) to toggle children.
  - **Drill-Down (Mobile):** Tap parent title to navigate into it.
  - **Selection:** Tap task row to open Edit modal.

### 4.4 The "Balance" View

- **UI:** List of Top-Level Items (excluding Inbox).
- **Controls:** Slider for "Desired %".
- **Feedback:** Visual bar for "Actual %" (computed from `effort` history). If Actual < Target, the row is highlighted to show it is "Starving" (boosting priority).

### 4.5 Task Editing (The "Details Modal")

Since there is no "selection" state on mobile, tapping any task text opens a full-screen modal (Mobile) or centered Popup (Desktop).

**Modal Contents:**

1. **Title:** Text input.
2. **Navigation & Hierarchy:**

- **Parent:** Read-only text showing current project.
- **"Move..." Button:** Opens a picker modal allowing the user to:
  1. **Select a new parent** (reparenting), and/or
  2. **Choose position among siblings** (reordering within the same or new parent).

  This is the **only mechanism for reorganizing tasks**—no drag-and-drop is required for MVP.

- **"Find in Plan" Button:** Closes the modal and navigates to the task's location in the Plan view.

3. **Status/Logic:**

- **Importance:** Slider (0.0 - 1.0).
- **Effort:** Segmented Control (1 | 3 | 5).

4. **Scheduling:**

- **Due Date:** Date Picker.
- **Lead Time:** Number input (Days).
- **Repeats:** Selector (None | Routinely | Calendar).

5. **Context:**

- **Place:** Dropdown picker linking to `RootDoc.places`.

6. **Notes:** Multi-line text area (Markdown).
7. **Footer Actions (The "Next Step" Workflow):**

- **"Add Sibling Task":** Creates a new task under the _same_ parent.
- **"Add Child Task":** Creates a new task under _this_ task.
- **"Delete Task"**

### 4.6 Visual Feedback & Sync Strategy

- **Priority differentiation:** Avoid showing raw scores. Order implies priority.
- **Sync State:** A subtle indicator (dot/cloud):
- _Green:_ Synced.
- _Yellow:_ Local changes, syncing...
- _Gray:_ Offline.

## 5. Future Enhancements (Out of Scope for MVP)

The following features are explicitly deferred to post-MVP releases. This is not a complete list, nor are these features guaranteed to ever be implemented:

1. **Snooze/Defer Task**: "Hide this task until a specific date." Would require a `snoozeUntil` field and priority suppression logic.
2. **Drag-and-Drop Reorganization**: Direct manipulation of task hierarchy in the Plan view. MVP uses the Move picker instead.
3. **Archive/Cleanup Completed Tasks**: Bulk deletion of tasks completed more than N days ago.
4. **Calendar View**: Visual timeline of tasks with due dates.
5. **Search**: Full-text search across task titles and notes.
6. **Tags/Labels**: Additional categorization beyond Places.

---

## 6. Implementation Guide

### 6.1 Type Definitions

See [automerge-schema.md](./automerge-schema.md) for schema documentation.

The canonical TypeScript implementation is in [`@mydoo/tasklens`](file:///Users/matt/src/mydoo/packages/tasklens):

- [`types.ts`](file:///Users/matt/src/mydoo/packages/tasklens/src/types.ts) — TypeScript interfaces
- [`schemas.ts`](file:///Users/matt/src/mydoo/packages/tasklens/src/schemas.ts) — Zod validation

### 6.2 Automerge Provider (`src/contexts/AutomergeContext.tsx`)

```typescript
import React, { createContext, useContext, useState, useEffect } from 'react';
import { Repo, DocHandle } from '@automerge/automerge-repo';
import { IndexedDBStorageAdapter } from '@automerge/automerge-repo-storage-indexeddb';
import { BrowserWebSocketClientAdapter } from '@automerge/automerge-repo-network-websocket';
import { RootDoc, ROOT_INBOX_ID, PLACE_ANYWHERE_ID } from '../types';

// Initialize Repo
const repo = new Repo({
  storage: new IndexedDBStorageAdapter(),
  network: [new BrowserWebSocketClientAdapter("ws://localhost:3030")],
});

interface AutomergeContextType {
  doc: RootDoc | undefined;
  handle: DocHandle<RootDoc> | undefined;
  ready: boolean;
}

const AutomergeContext = createContext<AutomergeContextType>({
  doc: undefined,
  handle: undefined,
  ready: false,
});

export const useAutomerge = () => useContext(AutomergeContext);

export const AutomergeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [handle, setHandle] = useState<DocHandle<RootDoc> | undefined>(undefined);
  const [doc, setDoc] = useState<RootDoc | undefined>(undefined);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const initDoc = async () => {
      // Simple strategy: persist the root handle URL in localStorage
      const rootHandleId = localStorage.getItem('rootHandleId');
      let myHandle: DocHandle<RootDoc>;

      if (rootHandleId) {
        myHandle = repo.find<RootDoc>(rootHandleId);
      } else {
        myHandle = repo.create<RootDoc>();
        localStorage.setItem('rootHandleId', myHandle.url);

        // Initialize Schema
        myHandle.change((d) => {
          d.tasks = {};
          d.places = {
            [PLACE_ANYWHERE_ID]: {
              id: PLACE_ANYWHERE_ID,
              name: "Anywhere",
              parentPlaceId: undefined
            }
          };
          d.rootTaskIds = [ROOT_INBOX_ID];

          // Create the Inbox Task
          d.tasks[ROOT_INBOX_ID] = {
            id: ROOT_INBOX_ID,
            title: "Inbox",
            notes: "System Inbox",
            parentId: undefined,
            childIds: [],
            status: 'active',
            createdAt: Date.now(),
            lastReviewTimestamp: Date.now(),
            importance: 0.5,
            effort: 1,
            dueDate: undefined,
            leadTimeDays: 0,
            repeatConfig: undefined,
            placeId: PLACE_ANYWHERE_ID
          };

          d.settings = {
            userName: "User",
            balanceSensitivity: 0.5
          };
        });
      }

      setHandle(myHandle);

      const v = await myHandle.doc();
      setDoc(v);
      setReady(true);

      // Listen for changes
      myHandle.on('change', (payload) => {
        setDoc(payload.doc);
      });
    };

    initDoc();
  }, []);

  return (
    <AutomergeContext.Provider value={{ doc, handle, ready }}>
      {children}
    </AutomergeContext.Provider>
  );
};

```
