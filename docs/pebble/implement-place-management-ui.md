---
id: issue-k0wyc7pgco
title: Implement Place Management UI
status: todo
priority: 20
created_at: 2026-03-02T14:59:34.839404005+00:00
tags:
  - task
---

Create a dedicated interface for users to manage their Places (CRUD operations).

Planning Steps:

1. Define 'PlaceManagementPage' view and add to router.
2. Implement a list view showing all existing Places from the store.
3. Create a 'PlaceEditor' component (dialog or inline) for creating and editing Places.
   - Fields: Name (string), Hours (string), Included Places (multi-select of other Place IDs).
4. Implement deletion logic with confirmation.
5. Add navigation entry (e.g., in Settings or Sidebar) to access the Places view.

Success Criteria:

- User can create, rename, and delete Places.
- User can configure 'Operating Hours' and 'Included Places' (hierarchy).
- Changes persist to the Automerge document.
- UI handles the 'Anywhere' built-in place correctly (e.g., non-deletable).
- Verified via BDD/Playwright tests.
