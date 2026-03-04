---
id: issue-ltucu6xlc
title: Add stream ordering/init test for persistence hooks
status: done
priority: 40
created_at: 2026-03-02T14:59:34.667721066+00:00
modified_at: 2026-03-04T01:07:26.327370581+00:00
resolved_at: 2026-03-04T01:07:15.617895090+00:00
tags:
  - task
---

Add a focused unit test to ensure initial heads are set before consuming handle.changes()/handle.persisted() updates.

Implemented in `crates/tasklens-ui/src/hooks/use_persistence.rs` by extending the test harness to capture memory/persisted head snapshots and adding `test_use_persistence_initializes_heads_before_stream_updates` to assert the first non-empty snapshot matches current document heads.

Verification:

- `cargo test -p tasklens-ui use_persistence`
- Result: 2 passed, 0 failed.
