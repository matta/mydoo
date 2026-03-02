---
id: issue-ti49iboxmw
title: Implement effective_credits aggregation from children in priority calculation
status: done
priority: 20
created_at: 2026-03-02T14:59:35.011052110+00:00
modified_at: 2026-03-02T14:59:35.017359892+00:00
resolved_at: 2026-03-02T14:59:35.017356645+00:00
tags:
  - task
---
Per algorithm.md §5.1: 'Parent's stored credits remain unchanged, but its effective_credits reflects the child's contribution.' Currently effective_credits is calculated as just the task's own decayed credits (priority.rs:211-212). It needs to aggregate decayed credits from all descendants during the post-order traversal phase. This affects feedback calculation which uses root effective_credits. Required for credit-attribution.feature.yaml scenarios: Parent Receives Effective Credits, Deep Hierarchy Propagation, Ancestor Decay Plus Child Attribution, Sibling Credits Independent.

## Close Reason

Superseded by mydoo-k9x (Implement effective_credits aggregation in priority.rs)
