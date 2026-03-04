---
id: issue-7pvk6zui91c
title: Replace settings dialog with non-modal panel
status: todo
priority: 9
created_at: 2026-03-04T03:24:17.674434766+00:00
modified_at: 2026-03-04T04:03:08.356625227+00:00
needs:
  - issue-sbm9yw59i5z
  - issue-lg8mkeb0sd8
  - issue-xqyyw005kvu
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

## Replan (2026-03-03)

- Execute after `issue-lg8mkeb0sd8` to reuse shared panel shell/chrome.
- Maintain `close-settings` test id and add `settings-panel` test anchor.
- Keep CSS implementation aligned with `docs/guidance/css.md`.
