# Row Component Implementation Plan

## Overview

Based on a review of the `tasklens-ui` CSS guidelines and the extensive use of `display: flex;` horizontal layout patterns across the codebase, the next most impactful layout primitive to implement is the **Row** component. Currently, there are dozens of CSS files (e.g., `task_row.css`, `app_navbar.css`, `doc_id_manager.css`) manually applying `display: flex`, various `gap` sizes, and `align-items`. Creating a standard standard layout primitive will unify these manual rules into a concise and consistent Dioxus component, similarly to the recent `Stack` migration.

## Implementation Steps

### 1. Create Layout Component (`Row`)

Create `crates/tasklens-ui/src/app_components/layout/row.rs`:

- Define Enums for properties mapping to CSS classes:
  - `RowGap`: `None`, `Xs`, `Sm`, `Md` (default), `Lg`, `Xl` (maps to app spacing tokens)
  - `RowAlign`: `Start`, `Center` (default), `End`, `Stretch`, `Baseline`
  - `RowJustify`: `Start` (default), `Center`, `End`, `Between`, `Around`
  - `RowWrap`: `NoWrap` (default), `Wrap`, `WrapReverse`
- Create the `Row` component using the `#[component]` macro with standard Dioxus attribute merging (`GlobalAttributes` and `div` extends).
- Add unit tests verifying attribute spread, CSS class composition, and virtual DOM logic, mirroring `stack.rs`.

### 2. Create CSS Module

Create `crates/tasklens-ui/src/app_components/layout/row.css`:

- Define `.row_root { display: flex; flex-direction: row; }`
- Define sizing classes like `.gap_sm { gap: var(--app_spacing_sm); }`
- Define alignment classes like `.align_center { align-items: center; }`
- Define justification classes like `.justify_between { justify-content: space-between; }`
- Define wrap classes like `.flex_wrap { flex-wrap: wrap; }`

### 3. Expose Row Component

Update `crates/tasklens-ui/src/app_components/layout/mod.rs`:

- Add `pub mod row;`
- Re-export `pub use row::{Row, RowGap, RowAlign, RowJustify, RowWrap};`

### 4. Integration & Refactoring (Iterative Phase)

Start replacing manual flex implementations in components. High-value targets identified:

- **`task_row.rs`** and **`priority_task_row.rs`**: Migrate `.row_root`, `.actions_container`, and related inner elements.
- **`app_navbar.rs`**: Standardize the top navbar horizontal layout and action groups.
- **`doc_id_manager.rs`**: Migrate header action rows and control panels.
- **`search_panel.rs`**: Clean up horizontal search inputs and filter pills.
- **Global `app.css`**: Eliminate `.app_row_cluster` occurrences.

### 5. Document Completion

- In `docs/guidance/css.md`, update the layout primitives list to mark `Cluster/Row` as "(Done)".
