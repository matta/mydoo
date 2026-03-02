---
id: issue-zef3diqus5a
title: Improve cycle healing to preserve parent hierarchy instead of promoting to root
status: todo
priority: 30
created_at: 2026-03-02T14:59:35.422992249+00:00
tags:
  - task
---
## Background

When concurrent moves create cycles (e.g., Replica A moves task-1 under task-2 while Replica B moves task-2 under task-1), the merged document contains a closed loop unreachable from the root hierarchy. The current fix in heal_structural_inconsistencies detects these unreachable tasks and breaks the cycle by promoting them to root tasks.

## Why This Is Suboptimal

Promoting cyclic tasks to root loses useful structural information. Consider tasks A and B that were both children of task P before the concurrent moves. After healing, one ends up at root level rather than remaining under P. This can be surprising to users who see tasks jump to the top level unexpectedly.

The ideal behavior would preserve the original parent relationship for at least one of the conflicting tasks, so the tree structure remains as close as possible to what either replica intended.

## Possible Approaches

1. Operation-level detection: Intercept move operations during merge and detect cycles before they are committed. This would require hooking into Automerge change application.

2. Schema change: Store move history or preferred parent metadata that survives merges, allowing smarter conflict resolution.

3. Last-write-wins on parent_id: Use Automerge conflict resolution metadata to determine which parent_id won and use that to inform cycle breaking.

4. Preserve one parent link: When breaking a cycle, instead of clearing parent_id entirely, traverse the cycle to find the oldest or most stable parent relationship and preserve it.

## Current Implementation

The expedient fix walks from roots to find reachable tasks, then promotes any unreachable tasks to root by clearing their parent_id. This guarantees invariants are restored but at the cost of hierarchy fidelity.

## Related Issues

- mydoo-1wb
- mydoo-cqu

---
*Imported from beads issue mydoo-cjj*
