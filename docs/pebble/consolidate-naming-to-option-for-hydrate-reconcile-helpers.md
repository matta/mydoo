---
id: issue-d8ecvsksp8j
title: Consolidate naming to 'option' for hydrate/reconcile helpers
status: done
priority: 20
created_at: 2026-03-02T14:59:35.959279814+00:00
modified_at: 2026-03-02T14:59:35.969016452+00:00
resolved_at: 2026-03-02T14:59:35.969012707+00:00
tags:
  - task
---
Rename all 'hydrate_optional_*' and 'reconcile_optional_*' functions to use the 'option' prefix (e.g., 'hydrate_option_f64', 'reconcile_option_as_maybe_missing') to be consistent with existing 'reconcile_option_string_as_text'. Update all call sites and #[autosurgeon] attributes.
