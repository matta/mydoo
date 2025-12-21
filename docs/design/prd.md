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

### 2.2 Data Schema (Monolithic Document)

**Concept: Normalized Data**
The Automerge document acts like a relational database. We do **not** store the tree structure directly (e.g., we do not nest `Task` objects inside other `Task` objects).

- **Storage:** All tasks exist in a single flat map (`tasks`), keyed by their ID.
- **Hierarchy:** The tree structure is reconstructed using reference IDs (`childIds` and `parentId`).
- **Benefit:** This allows O(1) lookups for any task, easy re-parenting (moving a task is just changing string IDs, not moving giant JSON objects), and efficient synchronization.

```typescript
// System Constants
const ROOT_INBOX_ID = 'system-inbox';
const PLACE_ANYWHERE_ID = 'system-place-anywhere';

interface RootDoc {
  /**
   * THE "DATABASE" TABLES
   * These maps are the single source of truth for all entities in the system.
   */

  /**
   * A flat map containing EVERY task in the system, regardless of status or depth.
   * - Includes: Active, Completed, Deleted, Inbox, and deeply nested sub-tasks.
   * - Key: The Task ID (UUID or Deterministic ID).
   */
  tasks: {[id: string]: Task};

  /**
   * A flat map containing ALL context places.
   * - Used to populate the "Context" dropdowns and filter lists.
   */
  places: {[id: string]: Place};

  /**
   * TOPOLOGY & ORDERING
   */

  /**
   * The entry points for the Plan view.
   * - Contains only the IDs of "Top-Level Items" (TLIs).
   * - Order: Visual order is determined by this array.
   * - CONFLICT HANDLING: Due to Automerge array merging strategies, this list may
   * temporarily contain duplicates. The "Healer" routine (Section 3.4)
   * sanitizes this by retaining only the *first* occurrence of any ID.
   */
  rootTaskIds: string[];

  /**
   * GLOBAL CONFIGURATION
   */
  settings: {
    userName: string;
    /**
     * Tuning parameter for the Balance Algorithm.
     * 0.0 = "Chill" (Balance deficits barely affect priority).
     * 1.0 = "Strict" (Balance deficits aggressively boost priority).
     */
    balanceSensitivity: number;
  };
}

interface Task {
  id: string;
  title: string;
  notes?: string; // Markdown supported. Undefined = no notes.

  /**
   * HIERARCHY / GRAPH CONNECTIONS
   */

  /**
   * The ID of the parent task.
   * - undefined: If this task is a Top-Level Item (TLI) listed in `RootDoc.rootTaskIds`.
   */
  parentId?: string;

  /**
   * Ordered list of children IDs.
   * - CONFLICT HANDLING: Like `rootTaskIds`, this may accumulate duplicates during merge.
   * The "Healer" routine ensures only the first occurrence of an ID is preserved.
   * - Empty array [] if leaf node.
   */
  childIds: string[];

  /**
   * STATE & LIFECYCLE
   */
  status: 'active' | 'completed' | 'deleted';

  createdAt: number; // Unix Timestamp (ms)

  /**
   * Used by the "Balance" algorithm to calculate Actual Effort history.
   * - undefined: If status is 'active' or 'deleted'.
   */
  completedAt?: number;

  /**
   * Used by the "Staleness" algorithm (see STALENESS.md).
   * - Updated on task creation, completion, snooze, or edit.
   * - A timestamp far in the past indicates the task is "stale".
   */
  lastReviewTimestamp: number;

  /**
   * SCORING FACTORS (See Algorithm Spec)
   */

  importance: number; // 0.0 (Trivial) to 1.0 (Critical).
  effort: number; // Weighting for Balance: 1 (Small), 3 (Med), 5 (Large)

  /**
   * SCHEDULING
   */

  dueDate?: string; // ISO Date (YYYY-MM-DD). Undefined = No deadline.
  leadTimeDays: number; // Days before due date to start showing. Must be >= 0. Ignored if dueDate is undefined.

  /**
   * REPETITION STRATEGY
   * - undefined: This is a one-off task.
   * - object: This task will regenerate upon completion.
   */
  repeatConfig?: {
    type: 'routinely' | 'calendar';
    intervalDays: number;

    /**
     * A UUID constant shared by all instances of this routine (e.g. "Laundry").
     * Used to prevent duplicates during merge conflicts.
     */
    seriesId: string;

    /**
     * Monotonically increasing counter (1, 2, 3...).
     * Used by the "Healer" routine to identify the latest version.
     */
    iterationIndex: number;
  };

  /**
   * CONTEXT
   * Links to `RootDoc.places`.
   * Defaults to PLACE_ANYWHERE_ID.
   */
  placeId: string;
}

interface Place {
  id: string;
  name: string;
  /**
   * Supports nesting (e.g., "Kitchen" is inside "Home").
   * - undefined: If this is a top-level context (e.g., "Home", "Work").
   */
  parentPlaceId?: string;
}
```

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

- **Visual Structure:** A flat list of tasks sorted by the Priority Algorithm.
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
  - **Checkbox:** Immediately sets `status = 'completed'` and triggers credit attribution (no staging).
  - **"Update/Refresh" Button:** Runs Repetition Logic (generates next instances) and triggers "Healer".
  - **Floating Action Button (Mobile):** Adds a new task directly to the **Inbox**.

### 4.3 The "Plan" View (Outline)

- **Structure:** Indented tree view. Infinite nesting supported.
- **Root Nodes:** Fixed "Inbox" node (pinned at top) + User TLIs.
- **Interaction:**
- **Expand/Collapse:** Chevrons (`>`) to toggle children.
- **Selection:** Tap to Edit.

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
- **"Move..." Button:** Opens a picker to select a new parent.
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

## 5. Implementation Guide

### 5.1 Type Definitions (`src/types.ts`)

```typescript
export const ROOT_INBOX_ID = 'system-inbox';
export const PLACE_ANYWHERE_ID = 'system-place-anywhere';

export type TaskStatus = 'active' | 'completed' | 'deleted';
export type RepeatType = 'routinely' | 'calendar';

export interface RepeatConfig {
  type: RepeatType;
  intervalDays: number;
  seriesId: string;
  iterationIndex: number;
}

export interface Task {
  id: string;
  title: string;
  notes?: string;

  /**
   * HIERARCHY:
   * 'parentId' is undefined if this is a Top Level Item (in rootTaskIds).
   */
  parentId?: string;
  childIds: string[];

  status: TaskStatus;
  createdAt: number;
  completedAt?: number;
  lastReviewTimestamp: number;

  importance: number; // 0.0 to 1.0
  effort: number; // 1 (Small), 3 (Medium), 5 (Large)

  dueDate?: string; // ISO Date string
  leadTimeDays: number;

  repeatConfig?: RepeatConfig;
  placeId: string;
}

export interface Place {
  id: string;
  name: string;
  parentPlaceId?: string;
}

export interface Settings {
  userName: string;
  balanceSensitivity: number;
}

export interface RootDoc {
  // The flat map of all tasks.
  // Tasks are NOT nested. Hierarchy is derived from parentId/childIds.
  tasks: {[id: string]: Task};

  places: {[id: string]: Place};

  // IDs of the top-level items (Goals/Categories)
  // NOTE: This array may contain duplicates due to merge conflicts.
  // The Healer algorithm sanitizes this by keeping only the first occurrence.
  rootTaskIds: string[];

  settings: Settings;
}
```

### 5.2 Automerge Provider (`src/contexts/AutomergeContext.tsx`)

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
