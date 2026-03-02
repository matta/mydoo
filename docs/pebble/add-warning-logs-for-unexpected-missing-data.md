---
id: issue-8kru8ptnshn
title: Add warning logs for unexpected missing data
status: todo
priority: 20
created_at: 2026-03-02T14:59:35.398979866+00:00
tags:
  - task
---
As per Gemini review: Use unwrap_or_else with tracing::warn! when looking up data that is expected to be present, to make issues detectable.
