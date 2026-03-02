---
id: issue-2zx695pw1ed
title: "Refactor types.rs: Separate persistent domain types from transient UI/utility types"
status: done
priority: 20
created_at: 2026-03-02T14:59:35.814417793+00:00
modified_at: 2026-03-02T14:59:35.824036735+00:00
resolved_at: 2026-03-02T14:59:35.824033272+00:00
tags:
  - chore
---
Split crates/tasklens-core/src/types.rs into two separate files:
1. Persistent types that are part of the Automerge document schema (using Hydrate/Reconcile).
2. Transient/Utility types like PriorityOptions and Context that are strictly *not* intended to be persisted in the Automerge doc, ensuring a cleaner separation of concerns between state storage and runtime logic.

## Close Reason

Refactored types.rs into persistent and transient modules.

---
*Imported from beads issue mydoo-qh3*
