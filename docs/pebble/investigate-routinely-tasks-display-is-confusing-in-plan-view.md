---
id: issue-o3awmljnu0j
title: "Investigate: Routinely tasks display is confusing in Plan view"
status: done
priority: 10
created_at: 2026-03-02T14:59:35.693801404+00:00
modified_at: 2026-03-04T02:37:33.826588240+00:00
resolved_at: 2026-03-04T02:37:33.826584203+00:00
tags:
  - task
---

Routinely tasks are currently represented as a single persistent task in the outline. When completed, they remain visible as 'done' (crossed out) but will eventually 'resurrect' in the future when their recurrence interval passes. This is confusing because they are visually indistinguishable from one-time completed tasks that stay 'done' forever.

Investigate:

1. How to visually distinguish these 'recurring templates' from standard completed tasks.
2. Whether to adopt the Things model where 'repetition specs' are distinct from 'todo' instances, or find a hybrid approach that keeps them in-place but clarifies their state.
3. Impact on the Plan view's mental model where 'done' usually means 'finished'.

## Related Issues

- mydoo-1wb

## Investigation Outcome (2026-03-03)

Implemented a hybrid clarification approach in Plan view:

- Pass `schedule_type` into `TaskRow` and display a `Routine` badge for pending `Routinely` tasks.
- For completed `Routinely` tasks, display `Repeats <relative date>` when next cycle information is available.
- Add unit tests for routine labeling logic in `task_row.rs`.

Result: completed routines are no longer visually equivalent to one-time done tasks, while keeping the current in-place singleton routine model.

## Follow-up

- issue-yz9p2rszx07: design whether to split recurring templates and concrete todo instances long-term.
