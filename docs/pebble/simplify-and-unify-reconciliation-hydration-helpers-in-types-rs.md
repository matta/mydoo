---
id: issue-3nnlsj3qsq
title: Simplify and unify reconciliation/hydration helpers in types.rs
status: done
priority: 20
created_at: 2026-03-02T14:59:34.909902328+00:00
modified_at: 2026-03-02T14:59:36.498079917+00:00
resolved_at: 2026-03-02T14:59:34.915614318+00:00
needs:
  - issue-b2r0izp6td
  - issue-czs7wkcyev
  - issue-ywcyd4k71g
  - issue-0716au3pra
tags:
  - epic
  - refactor
---
Leverage MaybeMissing and other built-in autosurgeon helpers to express hydration/reconciliation functions more consistently and idiomatically. Evaluate which custom helpers (like hydrate_f64/i64) can be replaced by standard library or autosurgeon-provided traits.
