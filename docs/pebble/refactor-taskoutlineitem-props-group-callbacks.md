---
id: issue-tca3j93pdu
title: Refactor TaskOutlineItem props (group callbacks)
status: todo
priority: 30
created_at: 2026-03-02T14:59:34.882286529+00:00
tags:
  - task
---
## Comments

**Matt Armstrong** (2026-02-08T01:30:48Z):
Investigated: concept applies to TaskRow (task_row.rs). It takes 6 separate EventHandler callback props. Grouping into a struct or providing via context would reduce prop drilling.
