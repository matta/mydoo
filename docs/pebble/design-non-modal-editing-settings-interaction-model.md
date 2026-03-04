---
id: issue-sbm9yw59i5z
title: Design non-modal editing/settings interaction model
status: done
priority: 8
created_at: 2026-03-04T03:24:17.646652604+00:00
modified_at: 2026-03-04T03:25:20.553143470+00:00
resolved_at: 2026-03-04T03:25:20.553140304+00:00
tags:
  - task
  - design
---

Define UX and architecture for replacing task editor + settings dialogs with non-modal surfaces.

Scope:

- Choose surface pattern (slide-in panel vs in-page card route) for TaskEditor and Settings.
- Define open/close behavior, keyboard interactions (Esc), and mobile responsiveness.
- Identify shared panel styles/components to avoid duplicated CSS.

Acceptance:

- A short design note in task body with chosen pattern and tradeoffs.
- Explicit constraints for testability (`data-testid`) and accessibility touch targets.

## Design Decision (2026-03-03)

### Chosen Pattern

Use a shared **non-modal slide-in panel shell** (right side on desktop, full-width sheet on narrow/mobile viewports) for both TaskEditor and Settings.

### Why This Pattern

- Keeps users in-context on Plan/Do pages instead of trapping focus in modal dialogs.
- Works for both heavy forms (TaskEditor) and utility configuration (Settings).
- Reuses one CSS/layout contract, reducing divergent UI behavior.

### Interaction Contract

- Open: explicit button actions only (Add Task/Edit/Settings).
- Close: close button, Cancel button, and `Escape` key.
- Background remains visible and scrollable; no backdrop/focus trap.
- Panel has independent internal scroll when content exceeds viewport.

### Mobile Behavior

- Below mobile breakpoint, panel spans full width and respects safe-area insets.
- Footer actions stay reachable without horizontal overflow.

### Accessibility/Testability Constraints

- Keep existing test ids (`close-settings`) and add stable panel anchors (`task-editor-panel`, `settings-panel`).
- Maintain >=44px actionable targets for panel controls.
- Preserve keyboard reachability for all form controls and close actions.

### Implementation Notes

- Introduce app-owned panel shell styles in component-local CSS modules (avoid vendored dialog overrides).
- Remove `DialogRoot` usage from TaskEditor and Settings entry flow.
- Keep MovePicker out of scope for this decomposition unless required by regressions.
