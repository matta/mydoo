---
id: issue-r6djbvbjo64
title: Balance Algorithm is disconnected from Priority Calculation
status: done
priority: 10
created_at: 2026-03-02T14:59:36.008388105+00:00
modified_at: 2026-03-02T14:59:36.018300486+00:00
resolved_at: 2026-03-02T14:59:36.018297127+00:00
tags:
  - bug
---
The 'Balance' algorithm (domain/balance.rs) is currently output-only and does not influence the core priority calculation (domain/priority.rs). This means strict balancing rules are not being applied to task scoring.
