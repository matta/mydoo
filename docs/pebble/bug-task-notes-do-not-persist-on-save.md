---
id: issue-14cfdw2tue
title: "Bug: Task notes do not persist on save"
status: done
priority: 10
created_at: 2026-03-02T14:59:35.306483706+00:00
modified_at: 2026-03-02T14:59:35.313968003+00:00
resolved_at: 2026-03-02T14:59:35.313964074+00:00
tags:
  - bug
---

## Close Reason

Fixed task notes persistence by adding missing 'notes' field to TaskUpdates in core and UI, and fixing label linkage in TaskEditor. Verified with E2E test.
