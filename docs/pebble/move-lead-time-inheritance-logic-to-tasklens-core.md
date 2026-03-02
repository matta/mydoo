---
id: issue-08k752sh8gd
title: Move lead time inheritance logic to tasklens-core
status: done
priority: 20
created_at: 2026-03-02T14:59:35.520376439+00:00
modified_at: 2026-03-02T14:59:35.528564838+00:00
resolved_at: 2026-03-02T14:59:35.528560608+00:00
tags:
  - task
---

The Plan view currently implements lead time inheritance during tree-walking. This non-trivial business logic should be moved to tasklens-core and verified through the conformance test fixture to ensure consistency across all views including Do mode.
