---
id: issue-23tptrui9aa
title: Fix inconsistent enum serialization in transient.rs
status: todo
priority: 20
created_at: 2026-03-02T14:59:35.998556903+00:00
tags:
  - task
---
Code review of the types refactor noted that ScheduleSource and PriorityMode have inconsistent serialization formats between serde (JSON) and autosurgeon (Automerge).

1. ScheduleSource: serde expects 'self'/'ancestor' (camelCase/renamed), but autosurgeon uses 'Myself'/'Ancestor' (PascalCase).
2. PriorityMode: serde expects 'do-list'/'plan-outline' (kebab-case), but autosurgeon uses 'DoList'/'PlanOutline' (PascalCase).

We should unify these to match the serde configuration while maintaining backward compatibility for hydration of existing documents.

---
*Imported from beads issue mydoo-xnz*
