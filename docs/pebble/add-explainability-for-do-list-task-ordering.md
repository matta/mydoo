---
id: issue-jc8oxtys0pm
title: Add explainability for Do list task ordering
status: todo
priority: 30
created_at: 2026-03-02T14:59:35.367518452+00:00
modified_at: 2026-03-02T14:59:36.677300821+00:00
needs:
  - issue-wzu1evbziam
tags:
  - task
---
The priority algorithm is complex, making it unclear why tasks appear in their current order. Users often wonder 'why is this task above that one?'

**Goals:**

1. **Static indicators**: Always-visible cues explaining why a task ranks high (e.g., inherited priority, balance algorithm boost, due date inflation)
2. **On-demand explanation**: A feature users can invoke to see a detailed breakdown of why a task appears at its current position

**Stretch goal**: Allow users to drag tasks in the Do list; the app then suggests property adjustments (on this task or others) to achieve the desired ordering. This requires significant work, so prioritize the 'why' explanation first.

**First step**: Brainstorm and prototype static and dynamic explanation approaches.
