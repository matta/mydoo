---
id: issue-k8gcg3x56i2
title: "Design: Handle ambiguous 'ready' tasks with completed children"
status: todo
priority: 30
created_at: 2026-03-02T14:59:36.089429490+00:00
tags:
  - task
---

## Problem

A task with completed children that is 'ready' and showing in the Do list is ambiguous:

1. **Ready to complete** - all children done, parent can be checked off
2. **Stuck** - needs planning attention to add new subtasks
3. **Evergreen container** - intended as a container for repeating tasks (never meant to be 'done')

This ambiguity is a natural consequence of the app design, but the app should help users distinguish these cases.

## Observed Behavior

Top-level tasks appear in the Do list when their children are complete, even when the user never intends to mark them done.

## Potential Solutions

1. **Visual distinction** - Show such tasks differently in the Do list (e.g., icon, color, grouping)
2. **New task property** - Add a feature akin to 'sequential' that means 'this task is a project that is never done and cannot be checked off as complete'. Working names: 'evergreen', 'ongoing', 'container', 'project-only'
3. **Investigate** - Should evergreen/container tasks even appear in the Do list at all?

## Questions to Resolve

- What should this property be called?
- Should such tasks be filterable from the Do list entirely?
- How should the UI indicate a task is 'stuck' vs 'ready to complete' vs 'evergreen'?

## Related Issues

- mydoo-1wb
