# Implementation Plan: Slider & Label Vertical Slice

This slice adopts the upstream `Slider` and `Label` components, replaces the legacy local `Slider` implementation, and clears `range*` debt in `TaskEditor` and `BalanceSlider`.

## Proposed Changes

### 1. Vendoring

- Update `crates/tasklens-ui/dioxus-vendor-components.toml` to include:
  - `slider`
  - `label`
- Run `cargo xtask dx-components vendor`.

### 2. Implementation

- **TaskEditor Refactor**:
  - Update `crates/tasklens-ui/src/app_components/task_editor.rs`.
  - Replace the `input[type="range"]` for `importance` and `effort` with the upstream `Slider` component (using `SliderTrack`, `SliderRange`, and `SliderThumb`).
  - Replace raw HTML `<label>` tags with the upstream `Label` component.
- **BalanceSlider Refactor**:
  - Update `crates/tasklens-ui/src/app_components/balance_slider.rs`.
  - Replace the `input[type="range"]` with the `Slider` component.
- **Legacy Removal**:
  - Delete `crates/tasklens-ui/src/components/slider/` as it's now fully superseded by the vendored version.

### 3. Documentation

- Update `docs/design/dioxus-components-migration.md`:
  - Move `Slider` and `Label` to "Adopted" status.
  - Update divergence summary.
- Update `docs/design/dioxus-components-migration.todo.md`:
  - Check off `range*` debt for `TaskEditor` and `BalanceSlider`.
  - Mark `Slider` and `Label` as completed in the vertical slices section.

## Verification Plan

### Automated Tests

- Run `pnpm verify` to check formatting and linting.
- Run `just test-e2e` to ensure the task editor and balance page still function correctly.

### Manual Verification

- Verify the new sliders in `TaskEditor` and `BalancePage` feel responsive.
- Inspect the DOM to ensure `Label` is correctly associated with inputs for accessibility.
