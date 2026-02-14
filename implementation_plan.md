# Implementation Plan: Toggle and Switch Vertical Slice

This slice adopts the upstream `Toggle`, `Switch`, and `ToggleGroup` components into `tasklens-ui`, replacing remaining checkbox-based toggle patterns and removing related DaisyUI debt.

## Proposed Changes

### 1. Vendoring

- Update `crates/tasklens-ui/dioxus-vendor-components.toml` to include:
  - `toggle`
  - `switch`
  - `toggle_group`
- Run `cargo xtask dx-components vendor` to pull these from the pinned registry.

### 2. Implementation

- **TaskEditor Refactor**:
  - Update `crates/tasklens-ui/src/app_components/task_editor.rs`.
  - Replace the `input[type="checkbox"]` for `is_sequential` (id `sequential-toggle`) with the upstream `Switch` component.
  - Remove the hardcoded `h-4 w-4` classes.
- **Audit Pass**:
  - Scan for other toggle-like patterns in `views/` and `app_components/`.
  - Replace with `Switch` or `Toggle` where appropriate.

### 3. Documentation

- Update `docs/design/dioxus-components-migration.md`:
  - Move `Toggle`, `Switch`, and `ToggleGroup` to "Adopted" status.
  - Update divergence summary.
- Update `docs/design/dioxus-components-migration.todo.md`:
  - Check off the Toggle/Switch tasks in Chunk E.

## Verification Plan

### Automated Tests

- Run `pnpm verify` to check formatting, linting, and unit tests.
- Run `just test-e2e` to ensure the task editor still correctly updates the sequential flag.

### Manual Verification

- Open the Task Editor.
- Verify the new `Switch` looks correct and functions properly.
- Verify accessibility (keyboard navigation for the switch).
