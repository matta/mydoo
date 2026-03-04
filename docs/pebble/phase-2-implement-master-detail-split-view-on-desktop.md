---
id: issue-inpgsgs0urg
title: "Phase 2: Implement List-Detail (Two-Pane) Layout on Desktop"
status: todo
priority: 10
created_at: 2026-03-04T04:44:50.285950062+00:00
modified_at: 2026-03-04T15:18:23.396544570+00:00
needs:
  - issue-zso28ewo1in
  - issue-g55zulfx3k4
tags:
  - task
  - ui
---

Implement List-Detail (Two-Pane) Layout on Desktop.

Rationale:
Once Phase 1 (Pure Full-Page Routing) is in place, we upgrade the desktop experience. Dioxus layouts and nested routing make this relatively straightforward, though we have to be careful about state. We do this after Phase 1 and the removal of all modal dialogs.

Scope:

- In Phase 1, we established canonical task routes (`/task/:id/edit` and `/task/new`).
- **Phase 1 routes are the permanent source of truth**. Do NOT rewrite the Dioxus router to use localized nested parameters (e.g. `#[route("/plan/:task_id")]`).
- Instead, implement a desktop listener or layout wrapper that monitors when the canonical `/task/:id/edit` route is active, and if the viewport width `> 768px`, renders the Editor pane adjacent to the List pane instead of unmounting the List.
- **Breakpoint Behavior is strictly additive**: On screens `< 768px` (mobile), the router should trigger the standard Phase 1 full page editor takeover.
- Phase 1 full-page routing remains the absolute canonical model. List-Detail (Two-Pane) layout is an environmental display rule over the same canonical routes.
- **Resize/Orientation Behavior**: If the editor is open and the viewport crosses the 768px breakpoint (e.g. due to tablet rotation or window resize), the UI must seamlessly transition layout. The list/editor should ideally remount or re-layout smoothly without losing user input state.

Acceptance:

- The URL still controls the state (back button always works perfectly on mobile to dismiss the editor).
- On desktop, clicking a task changes the URL but only swaps the right-hand side of the screen, creating a seamless list-detail (two-pane) layout.
- Under 768px, Phase 1 full-page flow triggers as normal (no CSS layering hacks).
- Dynamic Transition Safety: Resizing the window or rotating a tablet across the 768px breakpoint reorganizes the layout without losing any active context or state.
