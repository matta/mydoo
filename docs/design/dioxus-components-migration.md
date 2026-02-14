# Dioxus Components Migration: Active Plan

## Goals

- Use upstream Dioxus Labs components directly with minimal local edits.
- Keep vendored components close to upstream for low-friction syncs.
- Finish Tailwind removal after DaisyUI removal.
- Keep app-specific UI in `app_components` and vendored UI in `dioxus_components`.

## Tracking Policy

- Source of truth for this effort is this plan plus `dioxus-components-migration.todo.md`.
- Do not use `bd` for routine per-slice migration tracking unless explicitly requested.
- Keep both docs updated together when status changes.

## Completed Work (Condensed)

- Vendor workflow established around `cargo xtask dx-components vendor` with pinned registry/revision and `module_path = "src/dioxus_components"`.
- Module split established: app-owned UI under `app_components`; vendored upstream UI under `dioxus_components`; `components` retained only as compatibility shim.
- High-usage slices completed and integrated: Button, Input, Checkbox, Select, Dialog, Collapsible, Badge, Card, Progress, Textarea, Toggle/Switch, Label, Slider.
- Date/DateTime migrated to app-owned native wrappers (`DateInput`, `DateTimeInput`) with CSS modules.
- DaisyUI removal phase is complete (class debt cleared, plugin removed, `just verify` passed).

For detailed history on removed completed items, consult git history:

```bash
git log -- docs/design/dioxus-components-migration.md docs/design/dioxus-components-migration.todo.md
git show <commit>:docs/design/dioxus-components-migration.md
```

## Current State (Active)

- Tailwind runtime is still linked while Phase 2 remains open.
- `crates/tasklens-ui/src/components` still exists as a compatibility shim and should be removed after import migration completes.
- Adopted components with remaining alignment checks: Navbar and Popover.
- `dx-components-theme.css` still needs a pristine-upstream restore with app overrides isolated to `assets/app.css`.

## Slice Execution Guardrails

Each active slice must satisfy this definition of done:

1. Vendor one upstream component and integrate callsites.
2. Remove corresponding legacy implementation from `src/components` when no longer needed.
3. Remove related Daisy/Tailwind debt in touched callsites, or explicitly log deferred paths in the checklist.
4. Update migration docs in the same change.
5. Keep the active chunk focused on the highest-impact remaining debt.

## Module Boundary And Naming

- `crates/tasklens-ui/src/dioxus_components`: vendored upstream components.
- `crates/tasklens-ui/src/app_components`: app-owned components.
- `crates/tasklens-ui/src/components`: temporary compatibility shim only.

Required vendoring config:

```toml
# crates/tasklens-ui/dioxus-vendor-components.toml
module_path = "src/dioxus_components"

[registry]
git = "https://github.com/DioxusLabs/components"
rev = "<pinned-upstream-rev>"
```

## Component Acquisition Workflow

- Use `cargo xtask dx-components vendor` for all vendoring.
- Do not run `dx components add` directly in this repository.
- Keep local patches as follow-on commits after vendor snapshots.

Known risk to remember for future vendoring:

- `dx components add --git --rev` can mis-handle builtin dependency asset roots (upstream issue: https://github.com/DioxusLabs/dioxus/issues/5310).
- If a future component depends on builtin components with global assets, confirm copied assets explicitly after vendoring.

## CSS Strategy (Active)

- Keep vendored `dioxus_components` on upstream `style.css` + `document::Link` patterns.
- Use CSS modules for app-owned components where scoping helps.
- Keep `dx-components-theme.css` pristine upstream.
- Put app-specific layout/overrides in `assets/app.css`.

## Remaining Exit Criteria

Phase 1 (DaisyUI removal) is complete.

Phase 2 (Tailwind removal) remaining work:

1. Remove Tailwind inputs/outputs and stylesheet link:
   - `crates/tasklens-ui/tailwind.css`
   - `crates/tasklens-ui/assets/tailwind.css`
   - Tailwind link in `crates/tasklens-ui/src/main.rs`
2. Restore pristine upstream `dx-components-theme.css` and move local overrides to `assets/app.css`.
3. Run `just verify` successfully after Tailwind removal.

## Near-Term Priorities

1. Finish migrating remaining imports/implementations out of `crates/tasklens-ui/src/components`.
2. Re-sync Navbar and Popover to upstream formatting/CSS parity.
3. Complete Phase 2 Tailwind removal gates and re-verify.

## Checklist

See `dioxus-components-migration.todo.md` for the actionable checklist.
