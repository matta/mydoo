---
id: issue-wzu1evbziam
title: Priority inheritance causes unexpected ranking for neutral-priority descendants
status: done
priority: 10
created_at: 2026-03-02T14:59:35.640457157+00:00
modified_at: 2026-03-02T14:59:35.649320571+00:00
resolved_at: 2026-03-02T14:59:35.649317335+00:00
tags:
  - task
---
**Observed behavior:**
A top-level task 'Bikes' at priority 0.5 appeared at the top of the Do list. After adding a child task 'Clean up bike haven' also at priority 0.5 (the default), the child ranked in the middle of the Do list instead of at the same position as the parent.

**Expected behavior:**
Priority 0.5 should be 'neutral'—a chain of descendants all at 0.5 priority should rank equivalently to their greatest common ancestor, not decay downward.

**Possible cause:**
This may be unexpected behavior now that the default priority has changed to 0.5. The priority algorithm math may need adjustment to treat 0.5 as a true neutral value that doesn't compound/decay through inheritance.

**Action needed:**
Review the priority inheritance algorithm and adjust so that neutral-priority chains maintain equivalent ranking to their ancestor.

## Related Issues

- mydoo-jmi
