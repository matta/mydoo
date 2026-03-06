# Stack Layout Primitive Plan (`tasklens-ui`)

## Objective

Introduce a `Stack` layout primitive in `crates/tasklens-ui/src/app_components/layout` to standardize vertical flow spacing (`flex-direction: column` + tokenized `gap`) and eliminate recurring mystery margins, aligned with `docs/guidance/css.md` Sections 8.2, 8.3, 8.4, and 8.5.

## Why Stack First

- Vertical flow is the dominant layout pattern across `tasklens-ui` (`flex-direction: column` + `gap` repeated broadly).
- Spacing policy is currently distributed across many component/view CSS modules and some sibling `margin-bottom` rules.
- `Stack` localizes policy in one primitive, keeps RSX declarative, and enforces spacing tokens by construction.

## Scope

- In scope:
  - Add a new `Stack` component primitive.
  - Add colocated CSS module for stack classes.
  - Export the primitive from layout/app component modules.
  - Refactor `DoPage` task list composition to use `Stack` as the first migration target.
- Out of scope:
  - Full-repo migration of all vertical layouts in one PR.
  - New global utility classes in `assets/app.css`.
  - Introducing new spacing tokens.

## Proposed API

Implement a small, typed API (no mini-framework):

- `StackGap` enum:
  - `Xs`, `Sm`, `Md`, `Lg`, `Xl`
  - Maps directly to `--app_spacing_xs|sm|md|lg|xl`
- `StackAlign` enum:
  - `Stretch` (default), `Start`, `Center`, `End`
  - Maps to `align-items: stretch|flex-start|center|flex-end`
- `Stack` component props:
  - `gap: StackGap` (default: `Md`)
  - `align: StackAlign` (default: `Stretch`)
  - `children: Element`
  - `#[props(extends=GlobalAttributes)] #[props(extends=div)] attributes: Vec<Attribute>` for ergonomic passthrough (`id`, `data-*`, `aria-*`, extra classes)

## File Plan

- New: `crates/tasklens-ui/src/app_components/layout/stack.rs`
- New: `crates/tasklens-ui/src/app_components/layout/stack.css`
- Modify: `crates/tasklens-ui/src/app_components/layout/mod.rs`
  - `mod stack;`
  - `pub(crate) use stack::{Stack, StackAlign, StackGap};`
- Modify: `crates/tasklens-ui/src/app_components/mod.rs`
  - Re-export `Stack` (and enums if needed by call sites)
- Modify: `crates/tasklens-ui/src/views/do_page.rs`
  - Replace the `div` task-list wrapper class with `Stack { gap: StackGap::Sm, ... }`
- Modify: `crates/tasklens-ui/src/views/do_page.css`
  - Remove migrated `.task_list` layout class if no longer needed

## CSS Design

Use component-module classes with underscore naming and tokenized spacing only:

- Base class:
  - `.stack { display: flex; flex-direction: column; }`
- Gap classes:
  - `.gap_xs { gap: var(--app_spacing_xs); }`
  - `.gap_sm { gap: var(--app_spacing_sm); }`
  - `.gap_md { gap: var(--app_spacing_md); }`
  - `.gap_lg { gap: var(--app_spacing_lg); }`
  - `.gap_xl { gap: var(--app_spacing_xl); }`
- Align classes:
  - `.align_stretch`, `.align_start`, `.align_center`, `.align_end`

No hardcoded spacing values, no kebab-case class names, no inline concrete layout styles.

## Rust Implementation Notes

- Follow existing primitive pattern used by `PageContainer`:
  - `#[css_module(...)]` at module scope.
  - Build base attributes with `attributes!(div { class: ... })`.
  - Merge with user attributes via `merge_attributes`.
- Compose class names in pure helper functions for testability.
- Keep API intentionally small; defer `justify`, `wrap`, `as`, etc. to future primitives (`Row`/`Cluster`, `Grid`) unless a concrete call site requires them.

## Migration Strategy

1. Add the primitive and exports without changing behavior elsewhere.
2. Migrate `DoPage` task list wrapper to `Stack` as a low-risk pilot.
3. Verify visual parity.
4. Create follow-up issues for batch migrations in remaining views/components with repeated column+gap/margin patterns.

## Future Migration Targets (Planned)

The following locations have been identified as primary candidates for migration to the new `Stack` component to achieve uniformity and eliminate raw `div` + `flex-direction: column` duplication:

### 1. Task List Wrappers

Many views manually style a `div` wrapper for a list of task rows (often using a `.task_list` CSS class). These should be replaced by `<Stack gap={StackGap::Sm}>` (or appropriate gap):

- `crates/tasklens-ui/src/views/plan_page.rs` (task list wrapper)
- `crates/tasklens-ui/src/views/task_page.rs` (task list wrapper)
- `crates/tasklens-ui/src/views/score_trace_page.rs` (trace list)
- `crates/tasklens-ui/src/views/balance_page.rs`
  _Rationale: Centralizes the spacing policy for lists of rows across the entire application and allows us to delete the redundant `.task_list` CSS classes._

### 2. Legacy `app.css` Utilities (`.app_stack_4`, `.app_stack_6`)

- We must grep for usages of the legacy lobotomized owl selectors (`.app_stack_4` and `.app_stack_6`) in component Rust files.
- Replace `.app_stack_4` wrappers with `Stack { gap: StackGap::Lg }`.
- Replace `.app_stack_6` wrappers with `Stack { gap: StackGap::Xl }`.
  _Rationale: Eliminates deprecated lobotomized owl margins, moving us entirely to Flexbox `gap` semantics matching the CSS Style Guide._

### 3. Form Layouts and Settings Groupings

Complex components often hand-roll vertical layouts for form fields, labels, and setting groups:

- `crates/tasklens-ui/src/app_components/task_editor.rs`: The main `.task_editor_content` and sub-groups like `.sequential_label_group` use manual column flex.
- `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
- `crates/tasklens-ui/src/app_components/sync_indicator.rs`
- `crates/tasklens-ui/src/app_components/search_panel.rs`
  _Rationale: Forces our complex forms to adhere to the core tokenized spacing scale instead of driftings off-scale (e.g. replacing hardcoded `2.5rem` gaps with standard `Xl` tokens)._

### 4. Dioxus Vendored Component Internals

- Components like `Dialog`, `Popover`, and `Card` currently encode `flex-direction: column` inside their vendored CSS (`dialog/style.css`, `popover/style.css`, `card/style.css`).
- Going forward, avoid introducing dependencies from vendored code to app-owned layout primitives (`Stack`, `PageContainer`, etc.).
- Preferred approach: align vendored internals to the same shared design tokens used by app code (for spacing/layout consistency) while keeping vendored internals self-contained.
  _Rationale: Preserves ownership boundaries, reduces vendor-update friction, and still achieves visual consistency._

## Verification Plan

Automated checks:

- Add unit tests in `stack.rs` for class-composition mapping:
  - `StackGap::* -> gap_* class`
  - `StackAlign::* -> align_* class`
  - defaults produce `gap_md + align_stretch`
- Add a render smoke test (`VirtualDom::new`) for `Stack`.
- Run: `cargo test -p tasklens-ui`

Manual/runtime checks:

- Run `just dev`.
- Open the `Do` view and confirm task list spacing is unchanged.
- Confirm no new mystery margins were introduced and spacing still derives from app tokens.

## Risks and Mitigations

- Risk: Unexpected spacing shifts during pilot migration.
  - Mitigation: Limit first migration to `DoPage`, compare before/after behavior manually.
- Risk: API overgrowth.
  - Mitigation: Freeze initial API to `gap` + `align` only; extend only with demonstrated need.
- Risk: Class merge conflicts with consumer-provided classes.
  - Mitigation: Use attribute merge pattern already used by `PageContainer` and test merged class output.

## Definition of Done

- `Stack` primitive implemented and exported.
- `DoPage` uses `Stack` for vertical task-list flow.
- Unit tests and render smoke test pass.
- `cargo test -p tasklens-ui` exits `0`.
- Plan for subsequent migrations captured as follow-up issues.
