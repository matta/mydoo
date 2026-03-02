---
id: issue-gcv8qy6r5ws
title: App does not reconnect to sync server after disconnection
status: done
priority: 10
created_at: 2026-03-02T14:59:36.048323605+00:00
modified_at: 2026-03-02T14:59:36.058403447+00:00
resolved_at: 2026-03-02T14:59:36.058400056+00:00
tags:
  - task
---
**Problem:**
When the app disconnects from the sync server (e.g., due to tab inactivity or other conditions), it does not attempt to reconnect.

**Expected behavior:**
When the app becomes active again after disconnection, it should automatically attempt reconnection using exponential backoff.

**Notes:**
- Root cause of disconnection not yet characterized
- Possible triggers: tab going inactive, network interruption, server timeout
- Need to implement reconnection logic with exponential backoff strategy

## Related Issues

- mydoo-cqu
