# Repeating Tasks: Implementation Plan (Single Object Model)

**Status**: Adopted
**Date**: 2026-01-01

## 1. Core Philosophy: Single Object Identity

We are adopting a **Single Object Model** for repeating tasks.

- A "Laundry" task is a single persistent object in the database.
- It does not clone itself or create history entries (for MVP).
- It cycles between `Done` and `Pending` states based on time.

### Rationale

- **Edit Propagation**: Editing a "Done" task naturally edits the "Future" task because they are the same object.
- **Simplicity**: No complex series linking or ID management.
- **Data Purity**: Computing the "Do List" remains dynamic; we do not persist the list itself.

---

## 2. State & Lifecycle

### 2.1 The "Acknowledgment" Separation

We distinguish between the **Task State** (Business Logic) and the **View State** (UI Logic).

| Field            | Purpose                                     | Persistence                     |
| ---------------- | ------------------------------------------- | ------------------------------- |
| `status`         | Is the task conceptually complete?          | Persisted (`Pending` \| `Done`) |
| `isAcknowledged` | Has the user cleared it from the "Do" list? | Persisted (Boolean)             |

### 2.2 The Lifecycle Loop

1.  **Creation**:
    - Task created with `repeatConfig`.
    - `status: 'Pending'`, `isAcknowledged: false`.

2.  **Completion**:
    - User checks box.
    - `status` -> `Done`.
    - `isAcknowledged` -> `false` (remains visible in Do list).

3.  **Acknowledgment (UI Action)**:
    - User clicks "Update" (or similar).
    - `status` remains `Done`.
    - `isAcknowledged` -> `true`.
    - **Result**: Task disappears from Do List (filtered out). Task remains `Done` in Outline.

4.  **Regeneration (Time-Based Wake Up)**:
    - **Trigger**: Current time enters the `Lead Time` window of the _next_ occurrence.
    - **Action**:
      - Calculate next `dueDate`.
      - `status` -> `Pending`.
      - `isAcknowledged` -> `false`.
    - **Result**: Task reappears in Do List (as Pending) and unchecks itself in Outline.

---

## 3. Schema Changes

### 3.1 New Fields

No major structural changes (no new series objects). We refine the existing fields.

```typescript
// types.ts

interface Schedule {
  // 'Calendar' and 'Deadline' are deferred to Post-MVP (not in schema).
  type: 'Once' | 'Routinely';

  // Existing fields remain
  dueDate?: number;
  leadTime: number;
}

interface RepeatConfig {
  // Simplified config (no seriesId needed for Single Object)
  frequency: 'daily' | 'weekly' | 'monthly' | 'yearly';
  interval: number;

  // Anchor for Calendar mode calculations
  anchorDate?: number;
}
```

### 3.2 Removed/Deferred

- **Log Book / History**: We will NOT store a history of past completions for MVP.
- **Series IDs**: Not needed.

---

## 4. Implementation Logic

### 4.1 "Reconciler" Routine

The logic that wakes up tasks needs to run efficiently.

### 4.2 Handling 'Routinely' Logic (Resolved)

**Field Definition**: `lastCompletedAt` (Persisted)

- **Purpose**: Acts as the anchor point for the next occurrence of a floating ("Routinely") task.
- **Why**: Since Routinely tasks float relative to completion, we cannot calculate the next `dueDate` without knowing when the previous instance was finished.

**Usage Lifecycle**:

1.  **Write (User Action: Complete Task)**
    - User clicks checkbox.
    - System sets `status = 'Done'`.
    - System sets `lastCompletedAt = Date.now()`.
    - Task remains visible (crossed out) until acknowledged or woken up.

2.  **System Action: Reconciler**
    - The **Reconciler** constantly checks "Done" repeating tasks.
    - **Calculation**:
      ```typescript
      const periodMs = getPeriodMs(task.repeatConfig);
      const nextDueDate = task.lastCompletedAt + periodMs;
      const wakeUpTime = nextDueDate - task.schedule.leadTime;
      ```
    - **Decision**:
      - If `now >= wakeUpTime`, the task "Wakes Up".
      - System sets `status = 'Pending'`, `isAcknowledged = false`.
      - System updates `schedule.dueDate = nextDueDate`.

**Example**:

- Task: "Water Plants" (Every 3 days, lead time 1 day).
- User completes it on **Monday 10:00 AM**.
  - `lastCompletedAt` = Mon 10am.
- **Calculation**:
  - `interval` = 3 days.
  - `nextDueDate` = Mon 10am + 3 days = **Thursday 10:00 AM**.
  - `wakeUpTime` = Thu 10am - 1 day (lead time) = **Wednesday 10:00 AM**.
- **Result**: Task stays "Done" (or Hidden if acknowledged) until Wednesday 10am, at which point it reappears as "Pending" in the Do list.

---

## 5. Reconciler & Automerge Strategy

### 5.1 The Challenge of Automated Mutations

The Reconciler performs "administrative" mutations (Schema Migrations and Task Wake-ups) that are not initiated by user action. Since this logic runs locally on every client, there is a risk of "concurrent modification" where multiple devices attempt to perform the exact same maintenance simultaneously.

While Automerge handles conflicts gracefully, having multiple devices all generate a "Wake Up" mutation for the same task creates unnecessary history bloat.

### 5.2 Deterministic Schema Migrations

For **Schema Migrations** (e.g., converting `Recurring` -> `Routinely`), we should follow the **Deterministic Mutation Pattern** recommended by Automerge where possible.

**Strategy**:
We should use `Automerge.change` with a fixed **Actor ID** (e.g., `00000000-0000-0000-0000-MIGRATION-V1`) and a fixed **Timestamp** (e.g., `0`).

- **Why**: If Device A and Device B both run the migration, they will generate the _exact same binary change chunk_ (assuming they start from the same state). Even if they don't, using a fixed Actor ID helps group these "system" changes logically and ensures that subsequent merges are cleaner (idempotent operations converge).

**Note on "Hard-coded Byte Arrays"**:
The Automerge docs suggest hard-coding the change as a byte array. This works for _initialization_ (creating empty lists), but for _migration_ (iterating existing user data), we cannot pre-compute the byte array because the input state (`doc.tasks`) is unknown. Therefore, we will use the "Change on the fly with hard-coded ActorID" approach.

### 5.3 Implementation Logic (`runReconciler`)

We will implement a `runReconciler(handle)` function. This function must handle the distinction between "One-time Migration" and "Ongoing Logic".

```typescript
/**
 * administrative-mutations.ts
 *
 * Handles system-initiated changes using deterministic patterns where possible.
 */

const MIGRATION_ACTOR_ID = '00000000-0000-0000-0000-MIGRATION-V1';

export function runReconciler(handle: DocHandle<TunnelState>) {
  const doc = handle.docSync();
  if (!doc) return;

  // 1. Schema Migration: Recurring -> Routinely
  // We check purely for the presence of old data before attempting a write.
  const needsMigration = Object.values(doc.tasks).some(
    t => t.schedule?.type === 'Recurring',
  );

  if (needsMigration) {
    // Ideally, we apply this change with a specific ActorID/Time to be deterministic.
    // If the Repo/Handle API limits strictly to the current user's ActorID,
    // we will fall back to a standard change, relying on Automerge's convergence.
    handle.change(doc => {
      for (const task of Object.values(doc.tasks)) {
        if (task.schedule?.type === 'Recurring') {
          task.schedule.type = 'Routinely';
          // Best-effort backfill for lastCompletedAt if missing
          if (task.status === 'Done' && !task.lastCompletedAt) {
            task.lastCompletedAt = Date.now();
          }
        }
      }
    }); // TODO: Investigate passing { actor: MIGRATION_ACTOR_ID } if API permits.
  }

  // 2. Task Wake-Up (Daily Logic)
  // This must happen with the *current* time, so we use the standard actor.
  // We rely on the "status" check to ensure idempotency.
  handle.change(doc => {
    const now = Date.now();
    for (const task of Object.values(doc.tasks)) {
      if (task.status === 'Done' && task.schedule?.type === 'Routinely') {
        // ... calculate wakeUpTime ...
        // if (shouldWake) task.status = 'Pending';
      }
    }
  });
}
```

### 5.4 Integration

In `packages/tasklens/src/react/task-lens-provider.tsx`:

```typescript
useEffect(() => {
  if (!handle) return;
  // Run once on mount (load)
  runReconciler(handle);
}, [handle]);
```

---

## 6. Summary of Work (Phase 1)

1.  **Schema**:
    - Update `Schedule.type` enum.
    - Add `lastCompletedAt` (minimal state tracking).
    - Drop `seriesId` ideas.
2.  **Migration**:
    - Convert `Recurring` -> `Routinely`.
3.  **Logic**:
    - Implement `calculateNextDue()`.
    - Implement `wakeUpTask()` mutation.
