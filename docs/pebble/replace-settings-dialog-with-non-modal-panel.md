---
id: issue-7pvk6zui91c
title: "Phase 1: Migrate Settings to Full-Page Route Navigation"
status: todo
priority: 9
created_at: 2026-03-04T03:24:17.674434766+00:00
modified_at: 2026-03-04T04:44:49.136429085+00:00
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

- Add a new route to the Dioxus router: `/settings`.
- This route renders a dedicated `SettingsPage` component instead of overlapping `Settings` dialog/panel.
- Preserve document management actions. Implement an immediate edit UX. Most settings use immediate local commit; sync reconnect uses explicit Apply policy from issue-0k9xospwk90. Eliminate explicit save/cancel buttons where possible. The back button serves to close the page and changes should be auto-committed as they are made.
- Explicitly remove `role="dialog"` from the settings component as it is now a full page.

Acceptance:

- No dialog wrapper in settings/document management entry flow.
- Browser's back button navigates out of settings perfectly.
- Existing settings actions remain functional. Changing non-sync settings does not require Apply.
- Test Compatibility: Ensure the `close-settings` UI control contract remains fully functional for backward context. Introduce a stable page anchor selector (`settings-page`) for new E2E checks.
  - **In-App Parity Rule**: If browser history contains a prior in-app page, `close-settings` must perform a history back, and this behavior must be identical to pressing the native browser Back button.
  - **Deep-Link Exception**: If visited directly via deep-link (no in-app history), the parity rule is entirely exempted:
    1. The `close-settings` UI action must perform a history replace to the root (`/plan`) so that navigating back again does not re-open settings.
    2. The native browser Back button is permitted to exhibit standard host OS behavior (e.g., closing the tab completely).
