---
id: issue-9pshey4b6a9
title: >-
  Refactor get_prioritized_tasks API to separate schedule inheritance from
  filtering/sorting
status: todo
priority: 30
created_at: 2026-03-02T14:59:35.545216518+00:00
modified_at: 2026-03-03T02:56:11.130446214+00:00
needs:
  - issue-gpbrwdh01fd
tags:
  - task
---

The current get_prioritized_tasks API in tasklens-core does two things:

1. Computes effective schedule inheritance (due dates, lead times via tree-walking)
2. Filters/sorts tasks based on PriorityMode (DoList vs PlanOutline)

For the Plan view's use_schedule_lookup hook, we only need #1 - the schedule inheritance data. The PriorityMode parameter is irrelevant, making the API 'fat' and not fit for purpose.

**Recommended approach (Option A):** Create a dedicated compute_schedule_inheritance(state) -> HashMap<TaskID, ScheduleData> function in core that only does inheritance calculation.

**Pros:**

- Clean separation of concerns
- Plan view calls exactly what it needs
- No wasted work on filtering/sorting

**Cons:**

- May duplicate tree-walking logic (or requires extracting shared helper)
- Two code paths to maintain

**Context:** This came up during Gemini's code review of PR #209 (lead time inheritance centralization). Gemini suggested parameterizing PriorityMode, but we identified that the real issue is API design - passing a mode that doesn't affect the output indicates the API isn't well-suited for this use case.
