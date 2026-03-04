---
id: issue-zso28ewo1in
title: Update E2E tests for non-modal editor/settings flows
status: todo
priority: 11
created_at: 2026-03-04T03:24:17.668888098+00:00
modified_at: 2026-03-04T03:24:50.928551458+00:00
needs:
  - issue-7pv1d9mc600
  - issue-7pvk6zui91c
tags:
  - task
  - test
---

Update Playwright coverage for new non-modal surfaces.

Scope:

- Replace dialog-specific assumptions with panel/card selectors and interactions.
- Verify open/close and key workflows for task editing and settings management.

Acceptance:

- E2E tests pass via `just test-e2e` and `just verify`.
