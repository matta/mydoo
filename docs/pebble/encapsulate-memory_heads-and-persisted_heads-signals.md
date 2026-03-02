---
id: issue-r1c355kp1y0
title: Encapsulate memory_heads and persisted_heads signals
status: done
priority: 20
created_at: 2026-03-02T14:59:35.471410685+00:00
modified_at: 2026-03-02T14:59:35.479545671+00:00
resolved_at: 2026-03-02T14:59:35.479542332+00:00
tags:
  - task
---

Currently memory_heads and persisted_heads are exposed as bare Signal<String>. They should be encapsulated in a struct or similar abstraction so that read/write operations can be easily tracked using 'find references' features in IDEs.
