---
id: issue-2mxytu83va
title: Remove legacy fields from TunnelState
status: done
priority: 20
created_at: 2026-03-02T14:59:35.321637703+00:00
modified_at: 2026-03-02T14:59:35.329290296+00:00
resolved_at: 2026-03-02T14:59:35.329286597+00:00
tags:
  - task
---
Remove next_task_id and next_place_id from TunnelState struct in crates/tasklens-core/src/types.rs as they are legacy artifacts. Update doc_bridge.rs, default implementations, and test fixtures to reflect this change.

## Close Reason

done

---
*Imported from beads issue mydoo-bho*
