---
id: issue-j5fonnztwp
title: Enable aggregation-dependent compliance fixtures
status: done
priority: 20
created_at: 2026-03-02T14:59:34.968143545+00:00
modified_at: 2026-03-02T14:59:34.974046498+00:00
resolved_at: 2026-03-02T14:59:34.974043337+00:00
tags:
  - task
---

## Goal

Enable the credit attribution fixtures that depend on effective_credits aggregation from children.

## Context

- credit-attribution-aggregation.feature.yaml.pending contains 4 scenarios requiring aggregation
- These were split out during mydoo-38e.1.8.2 because aggregation wasn't implemented
- Once mydoo-k9x is complete, these can be enabled

## Key Files

- packages/tasklens/specs/compliance/fixtures/credit-attribution-aggregation.feature.yaml.pending (rename to .yaml)
- crates/tasklens-core/tests/compliance.rs (add to expected_files list)

## Scenarios to Enable

1. Parent Receives Effective Credits - child completion reflected in parent effective_credits
2. Deep Hierarchy Propagation - 3-level hierarchy aggregation
3. Ancestor Decay Plus Child Attribution - parent decay + child contribution
4. Sibling Credits Independent - sibling isolation + parent aggregation

## Implementation

1. Rename .pending file to .yaml
2. Add 'credit-attribution-aggregation.feature.yaml' to expected_files in compliance.rs
3. Run tests, fix any issues
4. Delete mydoo-59v (superseded by this work)

## Dependencies

- Depends on mydoo-k9x (aggregation implemented)

## Acceptance Criteria

- All 4 aggregation scenarios pass
- cargo test --test compliance passes
- No .pending files remain for credit attribution

## Close Reason

Enabled credit-attribution-aggregation.feature.yaml and verified with compliance tests.
