---
id: issue-13nadigqo
title: Refactor dispatch.rs to Full PersistedTask R-M-W Pattern
status: done
priority: 20
created_at: 2026-03-02T14:59:34.584927388+00:00
modified_at: 2026-03-02T14:59:34.589061434+00:00
resolved_at: 2026-03-02T14:59:34.589056982+00:00
tags:
  - task
---
Refactor dispatch.rs handlers to use full PersistedTask Read-Modify-Write pattern, eliminating field name strings and hydration function drift.

Current problem:
- Manual hydrate_f64/hydrate_optional_* calls with string field names
- reconcile_prop calls with string field names
- Drift risk: struct annotations can diverge from dispatch code

New pattern:
1. Shallow-hydrate DocIndex to get task ObjIds (O(1) lookup)
2. Full-hydrate individual PersistedTask structs
3. Modify typed Rust fields (compiler-enforced)
4. Reconcile full PersistedTask (uses struct's reconcile attributes)

Handlers to refactor:
- handle_complete_task (most complex, credit propagation)
- handle_update_task (sparse updates)
- handle_delete_task
- handle_move_task

Benefits:
- Single source of truth: PersistedTask definition
- Type safety: no string field names in dispatch logic
- Correct reconciliation: struct attributes define serialization

Depends on: mydoo-01z.2 (DocIndex Shallow Parse Type)
