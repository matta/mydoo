---
id: issue-lg8mkeb0sd8
title: Extract shared non-modal panel shell component
status: done
priority: 8
created_at: 2026-03-04T03:55:54.068113573+00:00
modified_at: 2026-03-04T04:22:49.086210714+00:00
resolved_at: 2026-03-04T04:22:49.086206459+00:00
needs:
  - issue-sbm9yw59i5z
tags:
  - task
  - ui
  - css
---

Create an app-level reusable panel shell for non-modal side panels/sheets, then migrate existing TaskEditor shell styles into it without changing editor-specific form styling.

Scope:

- Introduce shared component API in `app_components` (header/body/footer slots + close action + test id).
- Move generic shell/chrome CSS out of TaskEditor into shared panel styles.
- Keep TaskEditor behavior unchanged after extraction.
- Constrain CSS authoring to `docs/guidance/css.md` (tokens, touch targets, module usage).

Acceptance:

- Shared panel component is used by TaskEditor and ready for Settings migration.
- TaskEditor-specific styles remain local and focused on editor content only.
- No new hardcoded color tokens outside allowed guidance exceptions.

Scope refinement (2026-03-03): TaskEditor adoption is tracked separately in `issue-xqyyw005kvu`; this task focuses on introducing the shared component/CSS foundation.
