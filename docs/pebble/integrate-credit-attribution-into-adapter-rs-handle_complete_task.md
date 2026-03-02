---
id: issue-k9ae12zii01
title: Integrate credit attribution into adapter.rs handle_complete_task
status: done
priority: 20
created_at: 2026-03-02T14:59:35.579550480+00:00
modified_at: 2026-03-02T14:59:35.588185012+00:00
resolved_at: 2026-03-02T14:59:35.588181264+00:00
tags:
  - task
---
The handle_complete_task function in crates/tasklens-store/src/adapter.rs currently only sets status=Done and lastCompletedAt. It needs to also call the credit attribution logic from tasklens_core::domain::lifecycle::complete_task to apply decay and add credit_increment. This requires either: (1) hydrating the task, applying the domain function, and reconciling back, or (2) implementing the decay/increment logic directly in Automerge operations. The domain function complete_task in lifecycle.rs is already implemented. See algorithm.md §5.1.

## Close Reason

Superseded by mydoo-25x (Add credit attribution to adapter::handle_complete_task)
