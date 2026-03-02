---
id: issue-36km95hhdcq
title: Implement effective_credits aggregation in priority.rs
status: done
priority: 20
created_at: 2026-03-02T14:59:35.675998634+00:00
modified_at: 2026-03-02T14:59:35.685007860+00:00
resolved_at: 2026-03-02T14:59:35.685004121+00:00
tags:
  - task
---

## Goal

Aggregate children's effective_credits into parent's effective_credits during priority calculation.

## Context

- Algorithm.md §5.1: 'Parent's stored credits remain unchanged, but its effective_credits reflects the child's contribution'
- Currently priority.rs line 211-212 only calculates task's own decayed credits
- feedback.rs uses root.effective_credits for thermostat - needs subtree totals

## Key Files

- crates/tasklens-core/src/domain/priority.rs (modify evaluate_task_recursive)
- packages/tasklens/specs/compliance/fixtures/credit-attribution-aggregation.feature.yaml.pending (test scenarios)

## Implementation

In evaluate_task_recursive, after recursing into children (post-order position):

```rust
// After the for loop that recurses into children
let sum_children: f64 = child_indices
    .iter()
    .map(|&i| enriched_tasks[i].effective_credits)
    .sum();
enriched_tasks[task_idx].effective_credits += sum_children;
```

## Key Insight

- Phase 1 already calculates local decayed credits: effective_credits = credits \* decay_factor
- Post-order aggregation adds children's effective_credits to parent
- This happens regardless of visibility (feedback needs full subtree data)

## Dependencies

- Depends on mydoo-68j (compliance harness refactor complete)

## Acceptance Criteria

- Parent effective_credits = own_decayed + sum(children.effective_credits)
- thermostat.feature.yaml still passes
- Ready for aggregation fixtures to be enabled

## Close Reason

Implemented recursive effective_credits aggregation in priority.rs and aligned dispatch logic.
