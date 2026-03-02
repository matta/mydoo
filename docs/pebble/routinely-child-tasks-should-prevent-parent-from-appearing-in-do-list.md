---
id: issue-h1axt74hk
title: Routinely child tasks should prevent parent from appearing in Do list
status: done
priority: 10
created_at: 2026-03-02T14:59:34.659432463+00:00
modified_at: 2026-03-02T14:59:34.663527116+00:00
resolved_at: 2026-03-02T14:59:34.663523453+00:00
tags:
  - task
---
**Problem:**
A parent task (e.g., 'Health') that contains a routinely child task (e.g., 'Take daily allergy medicine') should never appear in the Do list because it always has pending work via its recurring child.

**Current behavior:**
The parent task appears in the Do list despite having routinely children.

**Expected behavior:**
Parent tasks with routinely children should be excluded from the Do list since they perpetually have actionable descendants.

**Investigation needed:**
- Review prd.md for specification of this behavior
- Review algorithm.md for how routinely tasks should affect parent visibility
- Add/verify YAML fixture test covering this scenario
- Ensure the scheduling algorithm properly accounts for routinely task inheritance

## Related Issues

- mydoo-m05
- mydoo-zup

---
*Imported from beads issue mydoo-1wb*
