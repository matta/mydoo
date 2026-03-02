---
id: issue-jmmu2k1ca9
title: "Optimize tasklens-ui E2E tests: Replace fixed pauses with observable waits"
status: done
priority: 0
created_at: 2026-03-02T14:59:35.226013243+00:00
modified_at: 2026-03-02T14:59:35.233408997+00:00
resolved_at: 2026-03-02T14:59:35.233405441+00:00
tags:
  - task
---
Scan tasklens-ui E2E tests for unnecessary fixed pauses (e.g., page.waitForTimeout) and replace them with faster, more reliable idioms such as waiting for specific DOM elements, network idle, or application state changes. This is critical for improving CI feedback loops and development velocity.

---
*Imported from beads issue mydoo-85b*
