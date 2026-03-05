---
id: issue-zso28ewo1in
title: Update E2E tests for route-based editor/settings flows
status: todo
priority: 11
created_at: 2026-03-04T03:24:17.668888098+00:00
modified_at: 2026-03-04T05:34:50.167673981+00:00
needs:
  - issue-7pv1d9mc600
  - issue-7pvk6zui91c
  - issue-0k9xospwk90
  - issue-xqyyw005kvu
tags:
  - task
  - test
---

Update Playwright coverage for new non-modal surfaces.

Scope:

- Rewrite E2E task language from "non-modal panel/card selectors" to route-navigation assertions.
- Add route-level test assertions to verify URL states and Back/Forward browser history behavior for in-app navigation flows.
- Ensure test coverage exists for route entry and exit from both Plan and Do views.
- **Sync Route Assertions:** Ensure clicking the `SyncIndicator` properly loads `/settings`. Assert that editing sync variables via the new `/settings` path immediately persists the local URL text while typing without triggering a reconnect thrash. Assert that network reconnects occur only on the explicit Apply action. Assert that invalid URL inputs safely fail to reconnect without crashing the sync routines.
- Replace unsaved-change handling requirement with test assertions for immediate persistence behavior: verify that "All task editor fields persist immediately on blur, explicit toggle, or navigation unmount." Keep unsaved-change checks only where a true draft model still exists.
- **Navigation Safety Assertions:** Add E2E that types into a text field and immediately navigates Back without blurring, proving no character loss. Add E2E that proves behavior of the immediate persistence mode under normal editing flow.
- **Search Return Assertions:** Test that clicking a search result opens the editor, and clicking Back returns to the underlying context safely (`/plan` or `/do`), dropping the search query history and NOT restoring dropped search text.
- **Find-in-Plan Return Assertions:** Add E2E that starts from Find-in-Plan, opens task editor, returns Back to `/plan`, and validates restored UI state via the `highlight` mapping (expanded tree path, viewport scroll, highlighted task row).
- **ViewContext Enum Assertions:** Verify URL query behavior strictly follows the Sum Type enum: `ctx=plan` and `ctx=do` must act as base types. `ctx=search` must include `return_to=[/plan|/do]`. `ctx=find_in_plan` must require `highlight=TaskID`.
  - **Exit Target Assertions:** Assert expected derived exit targets for each valid context: `ctx=plan` closes to `/plan`, `ctx=do` closes to `/do`, `ctx=search` closes to whatever `return_to` is provided, and `ctx=find_in_plan` implicitly closes to `/plan`.
- **Settings Routing Assertions:** Test that in-app navigation to `/settings` allows `close-settings` to return to the correct previous page using browser back-stack semantics.
- **Editor Routing Assertions:** Test that in-app navigation to the editor returns to the correct previous page using browser back-stack semantics. Add a precedence test where in-app history and enum-derived destination disagree (example: from `/plan`, open editor route with `ctx=search&return_to=/do`, then close): expected URL and visible page must both be `/plan` to prove history-back usage.
- **Deep-Link Non-Goal Assertions:** Do not add E2E coverage for direct deep-link entry (settings/editor, valid or malformed). These scenarios are unsupported user error and outside the test contract.
- Preserve existing coverage for create/save/delete/move/find, replacing dialog actions with full-page interactions.

Acceptance:

- E2E tests pass via `just test-e2e` and `just verify`.
- Full in-app URL contract behaves robustly, including backwards navigation to the correct context.
