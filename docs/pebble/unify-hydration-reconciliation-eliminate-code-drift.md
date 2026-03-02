---
id: issue-rtvhfbpl
title: "Unify hydration/reconciliation: eliminate code drift"
status: done
priority: 10
created_at: 2026-03-02T14:59:34.560346205+00:00
modified_at: 2026-03-02T14:59:36.119452667+00:00
resolved_at: 2026-03-02T14:59:34.566728208+00:00
needs:
  - issue-pgbw0v5b
  - issue-vsh012ck
  - issue-13nadigqo
tags:
  - task
---
## Problem

There is architectural dissonance between two hydration/reconciliation patterns:

1. Struct-level autosurgeon annotations on TunnelState, PersistedTask with custom hydrate/reconcile attributes (hydrate_f64, reconcile_f64, etc.)

2. Hand-written field-level code in adapter.rs that must manually use matching hydrators for partial updates

This creates code drift risk: if struct annotations change, adapter code must be updated manually. The compiler will not catch mismatches.

## Current Symptoms

- handle_complete_task must know to use hydrate_f64 for credits, hydrate_optional_f64 for creditIncrement
- These choices are duplicated from PersistedTask struct annotations
- Default autosurgeon::hydrate_prop fails on Int/F64 mixed values

## Desired State

Hydrate TunnelState, a single PersistedTask, or a single field, all with identical semantics from a single source of truth.

## Potential Solutions

1. Field accessor helpers: PersistedTask::hydrate_credits(doc, obj_id)
2. Wrapper types with Hydrate/Reconcile impls used consistently
3. Always hydrate full struct, mutate, reconcile (trading efficiency for correctness)

## Key Files

- crates/tasklens-core/src/types.rs (struct definitions with autosurgeon annotations)
- crates/tasklens-store/src/adapter.rs (hand-written partial hydration)

## Acceptance Criteria

- Single source of truth for hydration/reconciliation semantics per field
- Adapter code cannot accidentally use wrong hydrator
- Compiler or runtime catches mismatches if annotations change

---
*Imported from beads issue mydoo-01z*
