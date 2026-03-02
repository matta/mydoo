---
id: issue-a065w2q38a
title: Consider memoization for large OutlineTree lists
status: todo
priority: 30
created_at: 2026-03-02T14:59:34.877094291+00:00
tags:
  - task
---
## Comments

**Matt Armstrong** (2026-02-08T01:30:48Z):
Investigated: concept applies to PlanPage TaskRow rendering. The flattened_tasks memo caches the list, but every row still clones its PersistedTask and re-renders on any state change. Per-row memoization or virtualization could help for large trees.

---
*Imported from beads issue mydoo-38e.4.1*
