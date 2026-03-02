---
id: issue-67s2k9j87f5
title: Optimize repeat config event handlers
status: todo
priority: 20
created_at: 2026-03-02T14:59:35.391158952+00:00
tags:
  - task
---
Avoid unnecessary cloning in repetition-frequency-select and interval handlers by using with_mut and in-place updates (matches gemini-code-assist recommendation).
