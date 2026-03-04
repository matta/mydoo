---
id: issue-bjbkjh1w01z
title: Stop using modals for editing/creating tasks and sync settings
status: todo
priority: 10
created_at: 2026-03-02T14:59:35.748358745+00:00
modified_at: 2026-03-04T03:56:22.270378938+00:00
needs:
  - issue-sbm9yw59i5z
  - issue-7pv1d9mc600
  - issue-zso28ewo1in
  - issue-7pvk6zui91c
  - issue-lg8mkeb0sd8
tags:
  - task
---

Modals are fine for menus, but this app uses modals for editing and creating tasks, which is too heavyweight. Also don't like their use for setting up sync settings. The UX should be more web-like, where these things navigate or bring up 'cards' or slide-in 'panels' for these sorts of things.

## Decomposition (2026-03-03)

- issue-sbm9yw59i5z: design non-modal interaction model
- issue-7pv1d9mc600: migrate TaskEditor to slide-in panel
- issue-7pvk6zui91c: migrate settings UI to non-modal panel
- issue-zso28ewo1in: update E2E coverage for both flows

Execution order: design -> UI migrations -> E2E updates.

## Replan Addendum (2026-03-03)

Added intermediate extraction task:

- issue-lg8mkeb0sd8: extract shared non-modal panel shell component

Updated execution order: design -> TaskEditor migration -> shared panel extraction -> settings migration -> E2E updates.
