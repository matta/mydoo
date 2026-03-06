# Agents Dioxus Guide

This file is a concise agent-facing contract for Dioxus work in this
repository.

For deep framework quirks and debugging learnings, use:

- `docs/guidance/dioxus-gotchas.md`

For vendored component policy, use:

- `docs/design/dioxus-vendored-components-design.md`
- `docs/guidance/dioxus-vendored-customization.md`
- `docs/guidance/dioxus-vendored-styling.md`

## Core Rules

1. Target Dioxus `0.7` APIs (`Scope`, `cx`, and `use_state` are not valid).
2. Prefer app-owned wrappers/adapters to encode product semantics.
3. Avoid introducing dependencies from vendored component code
   (`crates/tasklens-ui/src/dioxus_components/**`) to app-owned code
   (`crate::app_components::*`, `crate::views::*`) unless an explicit,
   documented exception is approved.
4. When updating vendored components, prefer shared design token alignment over
   importing app-owned layout primitives into vendored internals.
5. For passthrough attributes on reusable primitives, prefer explicit
   `attributes!` + `merge_attributes` patterns where class merge behavior must
   be stable.
6. For CSS modules (`#[css_module]`), ensure at least one `Styles::<class>`
   value is passed directly in RSX/attributes so the stylesheet is not dropped
   by static tree-shaking.

## Runtime and Safety Notes

- Do not hold `Signal` / `RefCell` mutable borrows across `.await`.
- In WASM code paths, prefer explicit `expect("...")` over unannotated
  `unwrap()` for debuggability.
- Use `f64` for numeric values crossing FFI / Automerge boundaries.

## Testing Notes

- Prefer realistic user interactions in E2E tests (no synthetic events).
- Use semantic selectors (`data-testid`, domain attributes) over styling hooks.
