---
id: issue-yz9p2rszx07
title: Design recurring template/instance split for routine tasks
status: todo
priority: 20
created_at: 2026-03-04T02:37:23.735813352+00:00
modified_at: 2026-03-04T02:37:30.457920579+00:00
tags:
  - task
  - design
---

Evaluate whether mydoo should keep the current singleton in-place routine model or adopt a template+instance model (Things-style), with migration and CRDT implications.

Scope:

- Compare UX mental model in Plan/Do/Task views.
- Evaluate schema impacts in `docs/design/automerge-schema.md` and domain lifecycle logic.
- Propose a staged migration plan if split model is chosen.

Deliverable:

- Short design note with recommendation, tradeoffs, and rollout steps.
