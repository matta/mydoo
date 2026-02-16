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
3. Centralize app-specific customization in app-owned wrappers and styles.
4. Make vendor deltas explicit, minimal, and reversible.

## Non-Goals

1. Fork and evolve vendored components as a primary app extension surface.
2. Collapse app-owned and vendored ownership boundaries.
3. Optimize for short-term local hacks that raise long-term vendor drift.

## Ownership Boundaries

### Vendored-owned (pristine by default)

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`

### App-owned (customization surface)

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
4. Apply required vendor deltas as follow-on edits, not mixed into snapshot
   mechanics.
5. Validate with the repository quality gates for the change scope.
6. Document non-trivial vendor deltas and removal criteria in the PR.

## Vendor Delta Policy

Default policy: **pristine snapshot with selective minimal deltas**.

A vendor delta is acceptable only when:

1. app-owned wrappers/composition cannot express the need cleanly
2. the delta is minimal and scoped to the exact blocker
3. tests cover observable behavior of the change
4. a source-project issue/PR is opened for non-trivial deltas when practical
5. follow-up tracking exists to reconcile or remove the delta later

## Customization Delegation

For customization policy details, use:

- behavior/API/composition:
  `docs/guidance/dioxus-vendored-customization.md`
- styling/theme/class overrides:
  `docs/guidance/dioxus-vendored-styling.md`

This design doc defines ownership and workflow. The guidance docs define how to
customize safely within those boundaries.

## Review Checklist

1. Are changes in vendored paths necessary and minimal?
2. Was vendoring performed via `cargo xtask dx-components vendor`?
3. Are app-specific concerns kept in app-owned code where possible?
4. Are vendor deltas documented, tested, and tracked for reconciliation?
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
