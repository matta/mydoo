---
id: issue-cyy7fu3txj
title: Add credit attribution to adapter::handle_complete_task
status: done
priority: 20
created_at: 2026-03-02T14:59:34.671605273+00:00
modified_at: 2026-03-02T14:59:34.675511891+00:00
resolved_at: 2026-03-02T14:59:34.675508534+00:00
tags:
  - task
---

## Goal

Move credit attribution logic (decay + increment + timestamp update) into adapter.rs::handle_complete_task, which is the production code path for task completion.

## Context

- Algorithm.md §5.1 specifies: apply decay to existing credits, add credit_increment, update credits_timestamp
- Currently handle_complete_task only sets status=Done and lastCompletedAt
- A temporary lifecycle::complete_task was added but must be deleted - all mutations go through Store::dispatch

## Key Files

- crates/tasklens-store/src/adapter.rs (modify handle_complete_task)
- crates/tasklens-core/src/domain/lifecycle.rs (delete complete_task function and its tests)
- crates/tasklens-core/src/lib.rs (remove complete_task export)

## Implementation

Use the hydrate→mutate→reconcile pattern (like handle_refresh_lifecycle):

1. Hydrate the task from doc
2. Apply decay: decayed = credits \* 0.5^((current_time - credits_timestamp) / 604800000)
3. Add increment: credits = decayed + credit_increment.unwrap_or(0.5)
4. Update credits_timestamp = current_time
5. Set status = Done, lastCompletedAt = current_time
6. Reconcile back

## Acceptance Criteria

- lifecycle::complete_task deleted
- handle_complete_task applies credit attribution
- Existing adapter tests still pass
- cargo test --package tasklens-store passes
