---
id: issue-fwv6hp4bx3w
title: "Phase 2: Add E2E tests for List-Detail (Two-Pane) transitions"
status: todo
priority: 11
created_at: 2026-03-04T06:25:32.369168914+00:00
modified_at: 2026-03-04T15:18:23.408187338+00:00
needs:
  - issue-inpgsgs0urg
  - issue-g55zulfx3k4
tags:
  - task
  - test
---

Implement E2E test coverage for the List-Detail (Two-Pane) layout logic across the 768px layout transition boundary.

Rationale:
Phase 2 relies on dynamic transition safety where user input state and canonical URL states must survive layout shifting when the window resizes or device orientation changes.

Scope:

- Add test: editor open on desktop -> resize to mobile -> full-page takeover occurs without any state loss in the editor.
- Add test: editor open on mobile -> resize to desktop -> split-view layout occurs without any state loss in the editor.
- Add test: URL remains canonical and stable during all viewport resize transitions.

Acceptance:

- Playwright tests explicitly cover layout flipping back and forth successfully.
- Tests assert that editing a field on one layout, performing the resize transition, and finishing editing on the second layout correctly saves all input.
- Tests assert canonical route stability during each transition (`/task/:id/edit` remains active while layout mode changes) with no URL drift or accidental route fallback.
