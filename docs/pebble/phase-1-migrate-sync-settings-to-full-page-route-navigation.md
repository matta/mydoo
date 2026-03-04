---
id: issue-0k9xospwk90
title: "Phase 1: Migrate Sync Settings to Full-Page Route Navigation"
status: todo
priority: 9
created_at: 2026-03-04T04:51:11.456385500+00:00
needs:
  - issue-7pvk6zui91c
tags:
  - task
  - ui
---

Consolidate Sync Settings into the new Settings Full-Page Route.

Rationale:
To maintain consistency and reduce the number of floating overlays, the Sync Settings popover should be removed from the NavBar and its contents moved into the dedicated `/settings` route introduced in Phase 1.

Scope:

- Remove the `Sync Settings` popover from `SyncIndicator`.
- Expose the sync server URL input field inside the new `SettingsPage`.
- Update `SyncIndicator` to navigate to `/settings` when clicked, instead of toggling a local popover.
- **Commit Semantics:** Ensure immediate write of URL editing changes locally to persistence, but explicitly separate this local state storage from network transport reconnects. Local URL text still saves immediately.
- Establish a staged reconnect-trigger layer: Per-keystroke input commits to local storage immediately, but does NOT trigger a network-level disconnect/reconnect cycle per character. Sync reconnect uses an explicit Apply action.

Acceptance:

- Editing the sync server URL works from the `/settings` route using an immediate edit model locally.
- `SyncIndicator` acts as a status display and navigation link, with no overlay popover.
- Sync reconnect does not thrash the network on every keystroke during typing.
- Invalid URL inputs in settings do not crash or brick the sync routines.
