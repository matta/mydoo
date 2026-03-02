---
id: issue-a48pizxpgq8
title: Fix Visibility of Project with Completed Children
status: done
priority: 0
created_at: 2026-03-02T14:59:35.720761099+00:00
modified_at: 2026-03-02T14:59:35.730184402+00:00
resolved_at: 2026-03-02T14:59:35.730179853+00:00
tags:
  - bug
---
Tasks like 'Maple Tree Pruning' are not visible in the Do list. This task is a parent task (Project) with status Pending, but all its children are Done and Acknowledged. According to PRD Appendix A 'Container Visibility', 'if steps are hidden (sleeping), we show the Project to prevent Black Holes'. Currently, the project is hidden.

---
*Imported from beads issue mydoo-n7p*
