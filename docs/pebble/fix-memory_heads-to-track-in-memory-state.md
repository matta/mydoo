---
id: issue-3qdzodqwi1v
title: Fix memory_heads to track in-memory state
status: done
priority: 20
created_at: 2026-03-02T14:59:35.439208137+00:00
modified_at: 2026-03-02T14:59:35.447266196+00:00
resolved_at: 2026-03-02T14:59:35.447263005+00:00
tags:
  - task
---

Currently, memory_heads in use_persistence tracks the heads from the persisted store handle, which reflects the disk state rather than the pending in-memory state. This task is to update memory_heads. memory_heads should reflect the in-memory document state, not the persisted state.
