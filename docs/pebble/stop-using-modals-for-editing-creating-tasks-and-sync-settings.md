---
id: issue-bjbkjh1w01z
title: Stop using modals for editing/creating tasks and sync settings
status: todo
priority: 10
created_at: 2026-03-02T14:59:35.748358745+00:00
modified_at: 2026-03-04T15:18:23.419675683+00:00
needs:
  - issue-sbm9yw59i5z
  - issue-zso28ewo1in
  - issue-7pvk6zui91c
  - issue-xqyyw005kvu
  - issue-0k9xospwk90
  - issue-fwv6hp4bx3w
  - issue-inpgsgs0urg
  - issue-g55zulfx3k4
tags:
  - task
---

Modals are fine for menus, but this app uses modals for editing and creating tasks, which is too heavyweight. Also don't like their use for setting up sync settings. The UX should be more web-like, where these things navigate or bring up 'cards' or slide-in 'panels' for these sorts of things.

## Decomposition (2026-03-03)

Epic `needs` graph policy: The `needs` list tracks only the active implementation path; historical or abandoned tasks are removed from the graph instead of being preserved for history.
Pebble doc filename policy: keep existing filenames for ID/history continuity; no rename.

The previous slide-in panel extraction was abandoned in favor of a simpler, more robust full-page route implementation (Phase 1). This ensures excellent mobile UX, leverages Dioxus's strictly-typed Routing Enums to make invalid state transitions syntactically unrepresentable, and acts as a foundation for a desktop-only List-Detail (Two-Pane) implementation later (Phase 2).

- issue-sbm9yw59i5z: design non-modal interaction model (Completed)
- Phase 1 (Route Foundation):
  - issue-xqyyw005kvu: Migrate TaskEditor to Full-Page Route Navigation
  - issue-7pvk6zui91c: Migrate Settings to Full-Page Route Navigation
  - issue-0k9xospwk90: Migrate Sync Settings to Full-Page Route Navigation
  - issue-zso28ewo1in: Update E2E coverage for route-based editor/settings flows
- Phase 2 (Desktop Enhancement):
  - issue-g55zulfx3k4: Specify List-Detail transition E2E scenarios (pre-UI)
  - issue-inpgsgs0urg: Implement List-Detail (Two-Pane) Layout on Desktop
  - issue-fwv6hp4bx3w: Add E2E tests for List-Detail (Two-Pane) transitions

Execution order: UI route migrations (Phase 1) -> Route E2E updates -> Phase 2 transition spec (pre-UI) -> List-Detail (Two-Pane) UI (Phase 2) -> Phase 2 transition E2E.
