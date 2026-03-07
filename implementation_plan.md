# Row Component Implementation Plan (Single-Conversion Scope)

## Objective

Land the initial `Row` layout primitive and exactly **one** production conversion in the same change.

## Scope Guardrails

- In scope:
  - Implement `Row` primitive + CSS module.
  - Export `Row` through layout/app component modules.
  - Convert exactly one existing horizontal wrapper to `Row`.
- Out of scope:
  - Batch migration of all row-like wrappers.
  - Refactoring `task_row`, `app_navbar`, `search_panel`, or global utilities in this PR.

## Chosen Single Conversion

Convert only this wrapper:

- `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
  - First `div` using `Styles::row_layout` under "Current Document" block.
  - Replace with `Row { gap: RowGap::Sm, align: RowAlign::Center, ... }`.

Notes:

- Do not convert the second `Styles::row_layout` usage in the metadata subsection yet.
- Keep this PR intentionally narrow to validate primitive ergonomics with one real callsite.

## Implementation Steps

### 1. Create `Row` Primitive

Create:

- `crates/tasklens-ui/src/app_components/layout/row.rs`
- `crates/tasklens-ui/src/app_components/layout/row.css`

Initial API (minimal):

- `RowGap`: `None`, `Xs`, `Sm`, `Md` (default), `Lg`, `Xl`
- `RowAlign`: `Start`, `Center` (default), `End`, `Stretch`, `Baseline`
- `children: Element`
- Standard Dioxus attribute passthrough/merge (`GlobalAttributes` + `div`).

Defer justify/wrap until a concrete callsite requires them.

### 2. Export `Row`

Update:

- `crates/tasklens-ui/src/app_components/layout/mod.rs`
- `crates/tasklens-ui/src/app_components/mod.rs`

### 3. Apply the Single Conversion

Update:

- `crates/tasklens-ui/src/app_components/doc_id_manager.rs`

Optional cleanup (only if no remaining usage):

- Remove `display:flex` + horizontal gap/align concerns from the converted `.row_layout` CSS rule.

### 4. Tests and Validation

- Add focused unit tests in `row.rs` for class mapping/defaults and attribute spread.
- Run `just check`.
- Perform one manual visual check in Document Management:
  - Current document row keeps alignment and spacing parity.

## Post-Landing Roadmap (Planned)

After the initial Row primitive lands with one conversion, continue with iterative follow-up PRs.

### Progress Tracker

- [x] `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
  - Converted both previous `row_layout` wrappers to `Row`.
- [x] `crates/tasklens-ui/src/app_components/app_navbar.rs`
  - Converted navbar action cluster to `Row`.
- [x] `crates/tasklens-ui/src/app_components/search_panel.rs`
  - Converted search header controls wrapper to `Row`.
- [x] `crates/tasklens-ui/src/app_components/task_editor.rs` (iteration: Repeat Every)
  - Converted the `Repeat Every` controls wrapper from `app_row_cluster` to `Row`.
- [x] `crates/tasklens-ui/src/app_components/task_editor.rs` (iteration: Lead Time)
  - Converted the `Lead Time` controls wrapper from `app_row_cluster` to `Row`.
- [x] `crates/tasklens-ui/src/app_components/task_row.rs` (iteration: root wrapper)
  - Converted root task row wrapper to `Row`.
- [ ] `crates/tasklens-ui/src/app_components/task_row.rs` (remaining)
  - Migrate additional internal row-like wrappers as needed (for example actions cluster).
- [x] `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - Converted root task row wrapper to `Row`.
- [x] `crates/tasklens-ui/assets/app.css`
  - Removed unused `app_row_cluster` utility.

### High-Value Follow-Up Conversions

- `crates/tasklens-ui/src/app_components/task_row.rs`
  - Migrate row/action wrappers currently using manual flex alignment.
- `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - Align horizontal row wrappers with `Row`.
- `crates/tasklens-ui/src/app_components/app_navbar.rs`
  - Standardize navbar action clusters and horizontal groups.
- `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
  - Convert remaining `row_layout` usage(s) not included in phase 1.
- `crates/tasklens-ui/src/app_components/search_panel.rs`
  - Convert horizontal control/filter wrappers.
- `crates/tasklens-ui/assets/app.css`
  - Eliminate remaining legacy horizontal utility usage where `Row` is appropriate
    (for example `.app_row_cluster` callsites).

### API Expansion Triggers (Deferred)

- Add `RowJustify` only when a real callsite needs `justify-content` control.
- Add wrapping API only when a real callsite needs explicit `flex-wrap` behavior (Note: `doc_id_manager.rs` currently uses this for its button grid; prioritize when migrating that specific block).
- Keep the primitive minimal until concrete usage justifies expansion.

## Definition of Done (Phase 1: Initial Landing)

- `Row` primitive exists and is exported.
- Exactly one conversion is present (the selected `DocIdManager` row).
- `just check` passes.
- No additional row migrations are included in this change.

## Definition of Done (Row Initiative Complete)

- Planned follow-up conversions are completed in subsequent PRs.
- Legacy horizontal utility usage targeted by the roadmap is removed or justified.
- `crates/tasklens-ui/assets/app.css` is audited and unused classes are removed.
- `docs/guidance/css.md` is updated to mark Row/Cluster as done.
