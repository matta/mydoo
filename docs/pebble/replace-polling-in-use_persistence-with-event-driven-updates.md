---
id: issue-xcrg6k440hp
title: Replace polling in use_persistence with event-driven updates
status: done
priority: 20
created_at: 2026-03-02T14:59:35.455261175+00:00
modified_at: 2026-03-02T14:59:35.463310196+00:00
resolved_at: 2026-03-02T14:59:35.463307027+00:00
tags:
  - task
---
use_persistence.rs (and related hooks) currently polls for document changes every 100ms. This is inefficient. The persistence logic should react to messaging from samod to realize when the doc has been persisted instead of polling.

---
*Imported from beads issue mydoo-cqu.2*
