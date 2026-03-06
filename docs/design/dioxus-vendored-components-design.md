# Dioxus Vendored Components Design

This document defines how `tasklens-ui` vendors and maintains Dioxus component
code.

The historical migration effort is complete. This file now serves as the
long-lived design and operating policy for Dioxus Vendored code.

## Scope

This design governs:

- vendored component source in `crates/tasklens-ui/src/dioxus_components/**`
- vendored theme asset `crates/tasklens-ui/assets/dx-components-theme.css`
- vendoring configuration in
  `crates/tasklens-ui/dioxus-vendor-components.toml`
- source-project dependency pin coordination for `dioxus-primitives`

## Ubiquitous Language

- **Dioxus Vendored Code**: Code and assets copied from the Dioxus source
  project into this repository.
- **Source Project**: The external Dioxus components repository.
- **Vendor Snapshot**: A synced import produced by
  `cargo xtask dx-components vendor` at a pinned revision.
- **Vendor Delta**: Any local edits on top of a vendor snapshot.
- **App-Owned UI**: Components and styles maintained in `tasklens-ui` outside
  vendored paths.

## Goals

1. Keep Dioxus Vendored code close to source-project behavior and structure.
2. Preserve low-friction future vendor snapshot updates.
3. Prefer app-owned wrappers and styles for customization, but edit vendored
   code directly when it is the simpler, more maintainable solution.
4. Avoid gratuitous changes to vendored files that create merge noise without
   clear product value.
5. Maintain one-way dependencies: app-owned code may depend on vendored code,
   but vendored code should not depend on app-owned modules by default.

## Non-Goals

1. Collapse app-owned and vendored ownership boundaries without cause.
2. Optimize for short-term local hacks that raise long-term vendor drift.
3. Enforce a rigid pristine-only policy that makes simple changes artificially
   complex.
4. Treat vendored-to-app dependencies as a routine customization path.

## Ownership Boundaries

### Vendored-owned

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`

### App-owned (preferred customization surface)

- `crates/tasklens-ui/src/app_components/**`
- `crates/tasklens-ui/src/views/**`
- `crates/tasklens-ui/assets/app.css`

## Required Vendoring Configuration

`crates/tasklens-ui/dioxus-vendor-components.toml` is the source of truth for:

- `module_path = "src/dioxus_components"`
- source-project `registry.git`
- pinned source-project `registry.rev`
- component selection list

Do not change `module_path` without an explicit design decision.

## Vendoring Workflow

1. Ensure the working tree is clean or changes are intentionally isolated.
2. Update vendored code only via:
   - `cargo xtask dx-components vendor`
3. Review the vendor snapshot diff before any local customization.
4. Apply local edits as follow-on commits, not mixed into snapshot mechanics.
5. Validate with the repository quality gates for the change scope.

## Vendor Edit Policy

The upstream dioxus-components project is designed to be forked and edited.
Editing vendored code is acceptable when it is deliberate and product-driven.
Avoid gratuitous changes (reformatting, renaming, drive-by improvements) that
create merge noise without clear value.

Strong default: vendored source should not import or reference app-owned UI
modules (`crate::app_components::*`, `crate::views::*`). Prefer token-aligned
vendored-local edits or app-owned wrappers/composition around vendored
primitives.

Rare exceptions are allowed when product-critical and demonstrably simpler than
alternatives. Document exception rationale in the commit message and allowlist.

For detailed customization policy, see the guidance docs below.

## Customization Delegation

For customization policy details, use:

- behavior/API/composition:
  `docs/guidance/dioxus-vendored-customization.md`
- styling/theme/class overrides:
  `docs/guidance/dioxus-vendored-styling.md`

This design doc defines ownership and workflow. The guidance docs define how to
customize safely within those boundaries.

## Review Checklist

1. Are vendored edits deliberate and product-driven, not gratuitous?
2. Was vendoring performed via `cargo xtask dx-components vendor`?
3. Are app-specific concerns kept in app-owned code where reasonable?
4. Are vendored-to-app dependencies avoided (or explicitly justified as an
   approved exception)?
5. Do links and references use Dioxus Vendored terminology consistently?

## Historical Notes

The prior active migration checklist lived in
`docs/design/dioxus-components-migration.todo.md` and is now retired.

For historical audit details, use git history:

```bash
git log -- docs/design/dioxus-vendored-components-design.md docs/design/dioxus-components-migration.md docs/design/dioxus-components-migration.todo.md
git show <commit>:docs/design/dioxus-vendored-components-design.md
git show <commit>:docs/design/dioxus-components-migration.md
git show <commit>:docs/design/dioxus-components-migration.todo.md
```
