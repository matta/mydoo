---
id: issue-xofnrsxe3oj
title: Fix settings-button nested interactive markup in AppNavBar
status: done
priority: 80
created_at: 2026-03-05T01:02:55.714693974+00:00
modified_at: 2026-03-05T05:21:25.029899495+00:00
resolved_at: 2026-03-05T05:21:25.029895165+00:00
tags:
  - ui
  - a11y
---

Follow-up from code review: in `crates/tasklens-ui/src/app_components/app_navbar.rs`,
`settings-button` is currently rendered as a `Button` inside router `Link`, which creates nested interactive controls and can cause accessibility and behavior inconsistencies.

Expected fix:

- Replace nested interactive structure with a single interactive root (either link-styled control or button-driven imperative navigation).
- Preserve `data-testid="settings-button"` contract for Playwright.
- Add or adjust E2E checks for keyboard activation and click behavior parity.

Priority rationale: low-risk now, but should be resolved to avoid long-term accessibility debt.
