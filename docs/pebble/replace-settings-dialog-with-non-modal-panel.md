---
id: issue-7pvk6zui91c
title: "Phase 1: Migrate Settings to Full-Page Route Navigation"
status: in_progress
priority: 9
created_at: 2026-03-04T03:24:17.674434766+00:00
modified_at: 2026-03-04T21:17:42.225849388+00:00
needs:
  - issue-sbm9yw59i5z
tags:
  - task
  - ui
---

Migrate `Settings` UI to use Full-Page Route Navigation.

Rationale:
Following the two-phased approach, migrating to full page routing happens immediately to remove modal dialogs and create a robust mobile UX, skipping the intermediate shared panel implementation.

Scope:

- Add a new route to the Dioxus router: `/settings?[ViewContext Params]`.
- This route renders a dedicated `SettingsPage` component instead of overlapping `Settings` dialog/panel.
- Introduce or re-use a `ViewContext` enum to define the supported in-app query parameters (e.g., `?ctx=plan`) for provenance. Direct deep-link arrivals are unsupported user error and out of scope for deterministic behavior.
- Preserve document management actions. Implement an immediate edit UX. Most settings use immediate local commit; sync reconnect uses explicit Apply policy from issue-0k9xospwk90. Eliminate explicit save/cancel buttons where possible. The back button serves to close the page and changes should be auto-committed as they are made.
- Explicitly remove `role="dialog"` from the settings component as it is now a full page.

Acceptance:

- No dialog wrapper in settings/document management entry flow.
- Browser's back button navigates out of settings perfectly.
- Existing settings actions remain functional. Changing non-sync settings does not require Apply.
- Test Compatibility: Ensure the `close-settings` UI control contract remains fully functional for backward context. Introduce a stable page anchor selector (`settings-page`) for new E2E checks.
  - **Dioxus Navigation API Rule**: Tasklens UI code must use Dioxus Navigation APIs exclusively. `close-settings` uses `navigator().go_back()`.
  - **Deep-Link Non-Goal**: Direct entry to `/settings` via deep links is unsupported and untested. No fallback, replace, or recovery semantics are required.

Implementation Checklist:

- [x] Define `ViewContext` in `crates/tasklens-ui/src/router.rs` so `/settings` accepts approved return contexts via `ctx`; verify via `cargo build -p tasklens-ui`.
- [x] Add `Route::SettingsPage { ctx: Option<ViewContext> }` with `#[route("/settings?:ctx")]` in `crates/tasklens-ui/src/router.rs` so settings is a first-class routed page; verify via `cargo build -p tasklens-ui`.
- [x] Create `crates/tasklens-ui/src/views/settings_page.rs` with a top-level `data-testid="settings-page"` anchor and page heading so E2E can target full-page settings reliably; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Wire `Route::SettingsPage` into the app shell (imports and `Outlet::<Route>`) so `/settings` renders the new page component; verify via `cargo build -p tasklens-ui`.
- [x] Replace modal state toggling in `crates/tasklens-ui/src/app_components/app_navbar.rs` with route navigation to `/settings?ctx=plan` for Plan-origin settings entry so the Plan return context is explicit; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Replace modal state toggling in `crates/tasklens-ui/src/app_components/app_navbar.rs` with route navigation to `/settings?ctx=do` for Do-origin settings entry so the Do return context is explicit; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Move document-management UI from `crates/tasklens-ui/src/views/auth.rs` into `crates/tasklens-ui/src/views/settings_page.rs` without `DialogRoot`/`DialogContent` so settings no longer exposes `role="dialog"`; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Preserve existing document-management control test IDs (`close-settings`, `new-document-button`, `document-id-input`, `load-document-button`, `download-document-button`, `document-upload-input`) in `crates/tasklens-ui/src/views/settings_page.rs` so current workflows remain scriptable; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
- [x] Implement `close-settings` in `crates/tasklens-ui/src/views/settings_page.rs` to use `navigator().go_back()`; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Keep non-sync settings and document actions in `crates/tasklens-ui/src/views/settings_page.rs` as immediate-commit interactions with no Save/Cancel controls so page close is purely navigational; verify via `crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
- [x] Keep sync reconnect controls in `crates/tasklens-ui/src/app_components/sync_indicator.rs` on explicit Apply (`Save & Reconnect`) so issue-0k9xospwk90 behavior is unchanged; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/sync-settings.spec.ts`.
- [x] Add `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` and its fixture wiring so settings routing acceptance is enforced by E2E automation; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [x] Add a Plan-entry test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that opens settings from Plan and asserts the `settings-page` anchor and `ctx=plan`; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"Plan\"`.
- [x] Add a Do-entry test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that opens settings from Do and asserts the `settings-page` anchor and `ctx=do`; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"Do\"`.
- [x] Add a `Navigation Test` in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that verifies `close-settings` behavior.
- [x] Update settings helpers in `crates/tasklens-ui/tests/e2e/pages/plan-page.ts` to use `data-testid="settings-page"` instead of dialog role lookups so page-object abstractions match routed settings UX; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
