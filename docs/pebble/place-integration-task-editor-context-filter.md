---
id: issue-1t0yi63v9a
title: Place Integration (Task Editor, Context Filter)
status: todo
priority: 20
created_at: 2026-03-02T14:59:34.844821472+00:00
tags:
  - task
---
Integrate Place filtering into the primary application views.

Planning Steps:
1. Create a 'ContextPicker' component for the global header.
2. Add a 'selected_place' signal to the global UI state (app context).
3. Update 'use_do_list_tasks' hook to accept and apply the 'selected_place' filter.
4. Ensure 'TaskInput' on the Do page respects the active context when creating new tasks.
5. Verify that changing context updates the Do list immediately.

Success Criteria:
- Header displays a dropdown to switch between 'All Places' and specific Places.
- Selecting a Place filters the Do list to only show tasks associated with that Place.
- **Hierarchy Verification:** Selecting a Place (e.g., 'Home') MUST show tasks in included places (e.g., 'Desk').
- New tasks created while a filter is active are automatically assigned to that Place.
- Verified via BDD/Playwright tests.

---
*Imported from beads issue mydoo-38e.2.3*
