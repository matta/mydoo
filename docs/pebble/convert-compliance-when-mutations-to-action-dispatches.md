---
id: issue-o2mi2kkmpe
title: Convert compliance 'when' mutations to Action dispatches
status: done
priority: 20
created_at: 2026-03-02T14:59:35.162382400+00:00
modified_at: 2026-03-02T14:59:35.169421507+00:00
resolved_at: 2026-03-02T14:59:35.169417685+00:00
tags:
  - task
---
## Goal

Replace apply_mutation logic with Action dispatches for complete_tasks, delete_tasks, task_updates, etc.

## Context

- The 'when' block in fixtures applies mutations (complete task, delete task, update fields, advance time)
- Currently apply_mutation directly manipulates TunnelState
- Must convert to dispatch(Action::...) calls

## Key Files

- crates/tasklens-core/tests/compliance.rs (refactor apply_mutation)
- crates/tasklens-store/src/actions.rs (reference Action variants)

## Implementation

1. complete_tasks: dispatch(Action::CompleteTask { id, current_time })
2. delete_tasks: dispatch(Action::DeleteTask { id })
3. task_updates: dispatch(Action::UpdateTask { id, updates })
4. advance_time_seconds: Update test's current_time variable (not an Action)
5. update_credits: May need Action::UpdateTask with credits field

## Current Mutation struct fields

- advance_time_seconds: test state only
- update_credits: HashMap<String, F64OrString>
- task_updates: Vec<TaskUpdate>
- delete_tasks: Vec<String>
- complete_tasks: Vec<String>

## Dependencies

- Depends on mydoo-9ak ('given' mutations)
- Depends on mydoo-25x (credit attribution in CompleteTask)

## Acceptance Criteria

- apply_mutation removed or fully converted to dispatch calls
- All 'when' blocks work via Actions
- Existing fixtures still pass
