---
id: issue-o1i9k25d4o
title: Convert compliance 'given' mutations to Action dispatches
status: done
priority: 20
created_at: 2026-03-02T14:59:35.248021925+00:00
modified_at: 2026-03-02T14:59:35.255190039+00:00
resolved_at: 2026-03-02T14:59:35.255186429+00:00
tags:
  - task
---

## Goal

Replace the apply_task_input and apply_place_input functions with Action::CreateTask + Action::UpdateTask dispatches.

## Context

- The 'given' block in fixtures sets up initial state (tasks, places, current_time)
- Currently apply_task_input directly manipulates TunnelState
- Must convert to dispatch(Action::CreateTask) followed by dispatch(Action::UpdateTask) for field overrides

## Key Files

- crates/tasklens-core/tests/compliance.rs (refactor apply_initial_state, apply_task_input, apply_place_input)
- crates/tasklens-store/src/actions.rs (may need new Action variants)

## Implementation

1. For each task in 'given':
   - dispatch(Action::CreateTask { id, parent_id, title })
   - dispatch(Action::UpdateTask { id, updates: TaskUpdates { ... } }) for other fields
2. For places: May need Action::CreatePlace or use expensive_reconcile as fallback
3. For current_time: Store as test state, pass to Actions that need it
4. Handle nested children by processing parent first, then children with parent_id set

## Missing Action Support (may need to add)

- TaskUpdates may need: importance, credits, credit_increment, desired_credits, credits_timestamp, is_sequential
- May need Action::CreatePlace

## Dependencies

- Depends on mydoo-3dj (AppStore infrastructure)

## Acceptance Criteria

- apply_task_input removed or converted to use dispatch
- All 'given' blocks work via Actions
- Existing fixtures still pass
