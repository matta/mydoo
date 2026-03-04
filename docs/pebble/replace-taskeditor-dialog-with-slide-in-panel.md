---
id: issue-7pv1d9mc600
title: Replace TaskEditor dialog with slide-in panel
status: todo
priority: 9
created_at: 2026-03-04T03:24:17.649745875+00:00
modified_at: 2026-03-04T03:24:50.923284386+00:00
needs:
  - issue-sbm9yw59i5z
tags:
  - task
  - ui
---

Implement TaskEditor as non-modal UI surface per design task.

Scope:

- Remove dialog wrapper from TaskEditor create/edit flow in Plan/Do views.
- Preserve existing editor functionality (save/create/delete/move/find in plan).
- Keep or improve mobile usability and safe-area behavior.

Acceptance:

- No `DialogRoot` usage in TaskEditor path.
- Existing task editor interactions remain functional.
