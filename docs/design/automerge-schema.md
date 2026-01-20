# Automerge Document Schema

> **Abstraction Layer**: Automerge serves exclusively as the **storage and merge
> layer**. The rest of the application should be entirely decoupled from
> Automerge internals. Business logic, UI rendering, and algorithm calculations
> consume plain TypeScript objects—never Automerge proxies or documents
> directly.

This document specifies the **exact structure** of the Automerge CRDT document,
field by field.

---

## Top-Level Document: `TunnelState`

The root Automerge document is a single object with these fields:

| Field         | Type                    | Required | Description                                                |
| ------------- | ----------------------- | -------- | ---------------------------------------------------------- |
| `tasks`       | `Record<string, Task>`  | ✓        | Flat map of all tasks, keyed by task ID (UUID)             |
| `places`      | `Record<string, Place>` | ✓        | Flat map of all places, keyed by place ID (UUID)           |
| `rootTaskIds` | `string[]`              | ✓        | Ordered list of root-level task IDs (tasks with no parent) |

### Initialization

```javascript
{
  tasks: {},
  places: {},
  rootTaskIds: []
}
```

---

## Task Object

Each task stored in `tasks[id]` has the following fields:

### Required Fields (always present)

| Field               | Type                                   | Description                                         |
| ------------------- | -------------------------------------- | --------------------------------------------------- |
| `id`                | `string`                               | UUID. Matches the key in `tasks` map.               |
| `title`             | `string`                               | Task name/description                               |
| `status`            | `"Pending"` \| `"Done"` \| `"Deleted"` | Current state                                       |
| `importance`        | `number`                               | 0.0 to 1.0 priority weighting                       |
| `creditIncrement`   | `number`                               | Points awarded on completion (≥ 0)                  |
| `credits`           | `number`                               | Accumulated completion points                       |
| `desiredCredits`    | `number`                               | Target allocation for balancing (≥ 0)               |
| `creditsTimestamp`  | `number`                               | Unix ms when credits last modified                  |
| `priorityTimestamp` | `number`                               | Unix ms when priority recalculated                  |
| `isSequential`      | `boolean`                              | If true, children must complete in order            |
| `childTaskIds`      | `string[]`                             | Ordered list of child task IDs (empty `[]` if leaf) |
| `schedule`          | `Schedule`                             | Scheduling/deadline configuration (see below)       |

### Optional Fields (absent when not set)

| Field            | Type      | When present                                                                                           |
| ---------------- | --------- | ------------------------------------------------------------------------------------------------------ |
| `parentId`       | `string`  | Task ID of parent. **Absent** for root tasks.                                                          |
| `placeId`        | `string`  | Place ID for location context. **Absent** to inherit from parent.                                      |
| `isAcknowledged` | `boolean` | `true` when completed task has been acknowledged (hidden from Do list). **Absent** = not acknowledged. |

> **CRITICAL**: Automerge does not support `undefined` values. Optional fields
> are represented by **absence** (using the `delete` operator), not by setting
> to `undefined` or `null`.

### Schedule Sub-Object

The `schedule` field is always present and contains:

| Field      | Type                                      | Required | Description                                              |
| ---------- | ----------------------------------------- | -------- | -------------------------------------------------------- |
| `type`     | `"Once"` \| `"Routinely"` \| `"Deadline"` | ✓        | Temporal behavior selector (see below)                   |
| `leadTime` | `number`                                  | ✓        | Milliseconds before deadline to surface task             |
| `dueDate`  | `number`                                  | ✗        | Unix ms deadline. **Absent** if no deadline set.         |
| `period`   | `number`                                  | ✗        | Recurrence interval in ms. **Required** for `Routinely`. |

#### Schedule Type Semantics

**`"Once"` (Default)**

Non-recurring task. Temporal behavior is **polymorphic**:

- If an ancestor has a schedule with a deadline, this task inherits that
  constraint
- If no ancestor has a deadline, availability is determined by `importance` and
  `place` only
- On completion, task stays done (no regeneration)

**`"Routinely"` (Interval-Based / Floating)**

Maintenance tasks where next occurrence is relative to _completion_ time, not
scheduled time.

- **Recurrence**: `t_next = t_completion + period`
- **Initialization**: New routines default to `dueDate = now` (immediate
  urgency) until first completion
- **Priority Ramp** (driven by `leadTime`):
  - **Hidden**: `t_current < (dueDate - 2×leadTime)`
  - **Soon**: `(dueDate - 2×leadTime) ≤ t_current < (dueDate - leadTime)`
  - **Now**: `t_current ≥ (dueDate - leadTime)`
  - **Overdue**: `t_current > dueDate`

**`"Deadline"` (Hard Due Date)**

One-shot task with explicit deadline. Does **not** inherit from parent.

- `dueDate` is directly set (required for this type)
- Uses same priority ramp function as `Routinely`
- On completion, task stays done (no regeneration)

---

## Place Object

Each place stored in `places[id]` has:

| Field            | Type       | Required | Description                              |
| ---------------- | ---------- | -------- | ---------------------------------------- |
| `id`             | `string`   | ✓        | Matches the key in `places` map          |
| `hours`          | `string`   | ✓        | Opening hours specification (format TBD) |
| `includedPlaces` | `string[]` | ✓        | Place IDs contained within this place    |

### Reserved Place ID

`"Anywhere"` is a reserved virtual place. Tasks visible regardless of location
filter.

---

## Representing "Undefined" Values

Automerge cannot store JavaScript `undefined`. The codebase handles this by:

```typescript
// CORRECT: Delete the property
if (newTask.parentId === undefined) delete newTask.parentId;
if (newTask.placeId === undefined) delete newTask.placeId;
if (newTask.schedule.dueDate === undefined) delete newTask.schedule.dueDate;
if (newTask.schedule.period === undefined) delete newTask.schedule.period;

// WRONG: Automerge will error on this
task.parentId = undefined; // ❌ RangeError
```

When reading, absent fields are treated as `undefined` by the TypeScript layer.

---

## CRDT Merge Behavior

### Array Deduplication

Automerge arrays can accumulate duplicates during concurrent insertions. The
application sanitizes:

- **`rootTaskIds`** — First-occurrence-wins deduplication on load/sync
- **`childTaskIds`** — First-occurrence-wins deduplication on load/sync

### Object Maps

The `tasks` and `places` maps resolve concurrent writes **deterministically**
using Automerge's internal operation ordering (counter + actorId), not
wall-clock time. While a single "winning" value is returned by default,
Automerge preserves all conflicting values in a multi-value register accessible
via `Automerge.getConflicts()`.

---

## Computed Fields (NOT Stored)

The TypeScript `Task` interface includes computed properties populated by the
algorithm. These are **never written** to the Automerge document:

- `effectiveCredits`, `feedbackFactor`, `isContainer`, `isPending`
- `isReady`, `leadTimeFactor`, `normalizedImportance`
- `priority`, `visibility`

---

## Implementation Reference

| File                                                 | Purpose                                          |
| ---------------------------------------------------- | ------------------------------------------------ |
| [ops.ts](../../packages/tasklens/src/ops.ts)         | Mutation functions (shows exact field handling)  |
| [store.ts](../../packages/tasklens/src/store.ts)     | Automerge document wrapper                       |
| [types.ts](../../packages/tasklens/src/types.ts)     | TypeScript interfaces (includes computed fields) |
| [schemas.ts](../../packages/tasklens/src/schemas.ts) | Zod runtime validation                           |
