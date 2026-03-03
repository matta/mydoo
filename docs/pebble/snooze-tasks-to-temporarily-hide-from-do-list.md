---
id: issue-p2pqinlrnne
title: Snooze tasks to temporarily hide from Do list
status: todo
priority: 30
created_at: 2026-03-02T14:59:35.766881472+00:00
modified_at: 2026-03-03T02:56:11.518357325+00:00
tags:
  - task
---

Allow users to snooze a task, temporarily hiding it from the Do list until a specified time.

**Key behaviors:**

- Snoozing is distinct from scheduling a due date—'once' tasks can be snoozed without requiring a due date
- When snooze expires, the task reappears in the Do list and reverts to its previous status (loses 'snoozed' state)

**Edge cases & warnings:**

- Allow snoozing past due dates (e.g., snoozing a 'routinely' task for a month is valid)
- Warn when snoozing could cause a missed deadline (e.g., snoozing 'pay rent' past its due date is likely a mistake)

**Considerations:**

- How snooze interacts with inheritance and the priority algorithm
- UI for setting snooze duration (quick options like 'later today', 'tomorrow', 'next week', custom)
- What it means to snooze a parent task (does it snooze all children?)
- Visual indication of snoozed status in Plan mode—make it clear when a whole hierarchy is snoozed to avoid surprises when editing tasks

## Related Issues

- mydoo-1jv
