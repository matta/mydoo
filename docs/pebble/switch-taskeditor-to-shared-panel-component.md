---
id: issue-xqyyw005kvu
title: Switch TaskEditor to shared panel component
status: todo
priority: 9
created_at: 2026-03-04T04:02:57.199260682+00:00
modified_at: 2026-03-04T04:03:08.344954551+00:00
needs:
  - issue-lg8mkeb0sd8
tags:
  - task
  - ui
  - css
---

Migrate `TaskEditor` from local shell/chrome markup to the shared non-modal panel component introduced in `issue-lg8mkeb0sd8`.

Scope:

- Replace TaskEditor-specific outer shell with shared panel component usage.
- Preserve existing TaskEditor behavior (save/create/delete/move/find, close actions, Escape handling).
- Keep stable selectors (`task-editor-panel`, `close-task-editor`) for E2E compatibility.
- Keep CSS compliant with `docs/guidance/css.md` (module encapsulation, semantic tokens, touch targets).

Acceptance:

- TaskEditor uses shared panel component with no duplicated shell/chrome CSS.
- Existing TaskEditor interactions still pass verification.
- Any remaining TaskEditor CSS is editor-content specific only.
