---
id: issue-1q9ebjieli
title: Simplify 'swiss cheese' cfg logic in use_sync hook
status: todo
priority: 20
created_at: 2026-03-02T14:59:34.683686162+00:00
tags:
  - task
---

Refactor the sync hook to eliminate "swiss cheese" platform guards. The current implementation in `crates/tasklens-ui/src/hooks/use_sync.rs` is heavily fragmented by inline `#[cfg(...)]` checks. We must distill this complexity into elegant, platform-specific sub-modules or traits, restoring clarity to the hook core logic and paving the way for desktop support.
