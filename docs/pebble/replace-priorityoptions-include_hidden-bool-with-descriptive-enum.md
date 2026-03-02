---
id: issue-gpbrwdh01fd
title: Replace PriorityOptions.include_hidden bool with descriptive enum
status: todo
priority: 30
created_at: 2026-03-02T14:59:35.909758005+00:00
tags:
  - task
---

Change PriorityOptions.include_hidden from a boolean to a named enum (e.g., HiddenTasksPolicy { Include, Exclude }) to improve clarity at call sites and prevent 'boolean blindness'.
