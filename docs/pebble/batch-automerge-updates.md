---
id: issue-6ask27jbx3
title: Batch Automerge Updates
status: done
priority: 20
created_at: 2026-03-02T14:59:35.042578362+00:00
modified_at: 2026-03-02T14:59:35.049041360+00:00
resolved_at: 2026-03-02T14:59:35.049037864+00:00
tags:
  - task
---
## Close Reason

Already implemented: All actions (including SetBalanceDistribution which updates multiple tasks) execute within a single Automerge transaction via adapter::dispatch(). Writes are inherently batched.
