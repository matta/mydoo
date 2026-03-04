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
- Introduce or re-use a `ViewContext` enum to explicitly define allowed query parameters (e.g., `?ctx=plan`) and dictate the route's exit fallback behavior when history is empty or during deep-link arrivals.
- Preserve document management actions. Implement an immediate edit UX. Most settings use immediate local commit; sync reconnect uses explicit Apply policy from issue-0k9xospwk90. Eliminate explicit save/cancel buttons where possible. The back button serves to close the page and changes should be auto-committed as they are made.
- Explicitly remove `role="dialog"` from the settings component as it is now a full page.

Acceptance:

- No dialog wrapper in settings/document management entry flow.
- Browser's back button navigates out of settings perfectly.
- Existing settings actions remain functional. Changing non-sync settings does not require Apply.
- Test Compatibility: Ensure the `close-settings` UI control contract remains fully functional for backward context. Introduce a stable page anchor selector (`settings-page`) for new E2E checks.
  - **In-App Parity Rule**: If browser history contains a prior in-app page, `close-settings` must perform a history back, and this behavior must be identical to pressing the native browser Back button.
  - **Deep-Link Exception**: If visited directly via deep-link (no in-app history), the parity rule is entirely exempted:
    1. The `close-settings` UI action must perform a history replace to the target resolved by `ViewContext` (e.g. `/plan`) so that navigating back again does not re-open settings.
    2. The native browser Back button is permitted to exhibit standard host OS behavior (e.g., closing the tab completely).

Implementation Checklist:

- [ ] Define `ViewContext` and `SettingsQuery { ctx: ViewContext }` in `crates/tasklens-ui/src/router.rs` so `/settings` only accepts approved return contexts; verify via `cargo build -p tasklens-ui`.
- [ ] Add `Route::SettingsPage { query: SettingsQuery }` with `#[route("/settings?:ctx")]` in `crates/tasklens-ui/src/router.rs` so settings is a first-class routed page; verify via `cargo build -p tasklens-ui`.
- [ ] Create `crates/tasklens-ui/src/views/settings_page.rs` with a top-level `data-testid="settings-page"` anchor and page heading so E2E can target full-page settings reliably; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Wire `Route::SettingsPage` into the app shell (imports and `Outlet::<Route>`) so `/settings` renders the new page component; verify via `cargo build -p tasklens-ui`.
- [ ] Replace modal state toggling in `crates/tasklens-ui/src/app_components/app_navbar.rs` with route navigation to `/settings?ctx=plan` for Plan-origin settings entry so the Plan return context is explicit; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Replace modal state toggling in `crates/tasklens-ui/src/app_components/app_navbar.rs` with route navigation to `/settings?ctx=do` for Do-origin settings entry so the Do return context is explicit; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Move document-management UI from `crates/tasklens-ui/src/views/auth.rs` into `crates/tasklens-ui/src/views/settings_page.rs` without `DialogRoot`/`DialogContent` so settings no longer exposes `role="dialog"`; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Preserve existing document-management control test IDs (`close-settings`, `new-document-button`, `document-id-input`, `load-document-button`, `download-document-button`, `document-upload-input`) in `crates/tasklens-ui/src/views/settings_page.rs` so current workflows remain scriptable; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
- [ ] Implement `close-settings` in `crates/tasklens-ui/src/views/settings_page.rs` to perform history back when in-app history exists so behavior matches native browser Back from in-app entry; verify via an `In-App Parity Rule` test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Implement deep-link close fallback in `crates/tasklens-ui/src/views/settings_page.rs` to `history.replace` the route resolved from `SettingsQuery.ctx` when no in-app history exists so closing `/settings?...` cannot be reopened by immediate Back; verify via a `Deep-Link Exception` test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Define route-boundary behavior for missing/invalid `ctx` in `crates/tasklens-ui/src/router.rs` (normalize to `plan` fallback) so malformed deep links still exit deterministically; verify via `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Keep non-sync settings and document actions in `crates/tasklens-ui/src/views/settings_page.rs` as immediate-commit interactions with no Save/Cancel controls so page close is purely navigational; verify via `crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
- [ ] Keep sync reconnect controls in `crates/tasklens-ui/src/app_components/sync_indicator.rs` on explicit Apply (`Save & Reconnect`) so issue-0k9xospwk90 behavior is unchanged; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/sync-settings.spec.ts`.
- [ ] Add `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` and its fixture wiring so settings routing acceptance is enforced by E2E automation; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts`.
- [ ] Add a Plan-entry test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that opens settings from Plan and asserts the `settings-page` anchor and `ctx=plan`; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"Plan\"`.
- [ ] Add a Do-entry test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that opens settings from Do and asserts the `settings-page` anchor and `ctx=do`; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"Do\"`.
- [ ] Add an `In-App Parity Rule` test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that compares `close-settings` navigation with native browser Back behavior from in-app entry; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"In-App Parity\"`.
- [ ] Add a `Deep-Link Exception` test in `crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts` that loads `/settings?ctx=plan`, clicks `close-settings`, and asserts replace-to-fallback semantics; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/settings-route-navigation.spec.ts -g \"Deep-Link Exception\"`.
- [ ] Update settings helpers in `crates/tasklens-ui/tests/e2e/pages/plan-page.ts` to use `data-testid="settings-page"` instead of dialog role lookups so page-object abstractions match routed settings UX; verify via `just test-e2e -- crates/tasklens-ui/tests/e2e/specs/document-switching.spec.ts`.
