---
id: issue-pgbw0v5b
title: Manual TunnelState Hydration Module
status: done
priority: 20
created_at: 2026-03-02T14:59:34.570341377+00:00
modified_at: 2026-03-02T14:59:34.573987710+00:00
resolved_at: 2026-03-02T14:59:34.573982721+00:00
tags:
  - task
---
Remove autosurgeon decorators from TunnelState struct. It is too expensive to hydrate the entire TunnelState to/from the doc. Create a new module with manual hydration/reconciliation functions that we control directly.

Key changes:
- Remove #[derive(Hydrate, Reconcile)] from TunnelState
- Create crates/tasklens-core/src/domain/doc_hydration.rs (or similar)
- Implement manual hydrate_tunnel_state() and reconcile_tunnel_state() functions
- These functions have full control over granularity and can be optimized

This is foundational work for the DocIndex pattern.
