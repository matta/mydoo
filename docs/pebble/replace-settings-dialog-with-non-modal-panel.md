---
id: issue-7pvk6zui91c
title: Replace settings dialog with non-modal panel
status: todo
priority: 9
created_at: 2026-03-04T03:24:17.674434766+00:00
modified_at: 2026-03-04T03:24:50.926726349+00:00
needs:
  - issue-sbm9yw59i5z
tags:
  - task
  - ui
---

Implement settings/document management as non-modal UI surface per design task.

Scope:

- Replace settings dialog surface from app navbar with panel/card UX.
- Preserve document management actions and `close-settings` test id.

Acceptance:

- No dialog wrapper in settings/document management entry flow.
- Existing settings actions remain functional.
