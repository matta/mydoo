---
id: issue-4zzyumb3f8
title: Create compliance harness AppStore infrastructure
status: done
priority: 20
created_at: 2026-03-02T14:59:34.898648678+00:00
modified_at: 2026-03-02T14:59:34.904331611+00:00
resolved_at: 2026-03-02T14:59:34.904327827+00:00
tags:
  - task
---

## Goal

Refactor the compliance test harness to use AppStore with Automerge documents instead of directly manipulating TunnelState.

## Status

DONE with caveat: The current implementation works but has a layering violation where tasklens-core dev-depends on tasklens-store. This will be resolved by:

- mydoo-qkt: Lift Action enum to core
- mydoo-9jt: Lift run_action to core
- mydoo-ouo: Remove store dependency from compliance

## Implementation Summary

- Added tasklens-store as dev-dependency
- Created ComplianceStore shim using AutoCommit directly
- Refactored run_scenario to use store.hydrate() and store.expensive_reconcile()
- Mutations use store.dispatch(Action::DeleteTask) and store.dispatch(Action::CompleteTask)
- Made adapter::run_action public (temporary, will be reverted when lifted to core)
- Removed complete_task_inline and recursive_delete_task helpers

## Key Files Changed

- crates/tasklens-core/Cargo.toml (added tasklens-store dev-dep)
- crates/tasklens-core/tests/compliance.rs (major refactor)
- crates/tasklens-store/src/adapter.rs (made run_action pub)

## Acceptance Criteria

✓ Compliance harness uses Automerge documents, not raw TunnelState
✓ All existing passing fixtures still pass
✓ cargo test --test compliance passes
