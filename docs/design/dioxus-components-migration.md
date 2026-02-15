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
- `crates/tasklens-ui/src/components` compatibility shim has been removed (TW1 complete).
- Generated `crates/tasklens-ui/assets/tailwind.css` is `395` lines as of 2026-02-15 after TW5a `@source` narrowing.
- Output still contains Tailwind preflight/reset and properties layers because `crates/tasklens-ui/tailwind.css` imports the Tailwind runtime.
- Remaining app-owned utility dependencies tracked for TW5a are now removed from app-owned Rust callsites; continue auditing new callsites for regressions.
- `@apply` has been removed from `crates/tasklens-ui/tailwind.css`; runtime dependency is now isolated to explicit reset/theme/base decisions.
- Compiled output includes likely extraction noise selectors (for example `.container` and `.table`) with no matching app-owned Rust callsites.
- `dx-components-theme.css` still needs a pristine-upstream restore with app overrides isolated to `assets/app.css`.
- Tailwind utility usage matches in app-owned source: materially reduced after page-shell migration.

## Slice Execution Guardrails

Each active slice must satisfy this definition of done:

1. Keep one coherent concern per PR (single migration slice with explicit in-scope files).
2. For slices touching legacy imports, migrate callsites off `src/components` and remove the no-longer-needed shim module in the same or immediately following slice.
3. Remove related Tailwind class debt in touched callsites, or explicitly log deferred paths in the checklist.
4. Update migration docs in the same change, including any baseline/metric updates used by Phase 2 gates.
5. Keep the active chunk focused on the highest-impact remaining debt and avoid broad cross-app rewrites.

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
- For utility-class replacements in app-owned Rust callsites, prefer point-of-use classes in each component's scoped CSS module instead of adding new shared utility helpers to `crates/tasklens-ui/assets/app.css`.
- Keep `dx-components-theme.css` pristine upstream.
- Put app-specific layout/overrides in `assets/app.css`.

## Tailwind Findings (2026-02-15)

- Tailwind output dropped from the earlier ~1196-line planning baseline to 395 lines after TW5a `@source` narrowing and `@apply` removal, but is not yet removable.
- The largest remaining CSS bulk appears to be Tailwind preflight/reset and `@property` scaffolding, not app utility selectors.
- Before final removal, we need a canary slice that proves app behavior does not implicitly rely on preflight defaults.
- Runtime removal should follow only after:
  - utility dependencies (`sr-only`, `size-5`) are replaced with app-owned classes,
  - `@apply` is removed from Tailwind input,
  - required reset behavior is explicitly owned in `assets/app.css`.

## Remaining Exit Criteria

Phase 1 (DaisyUI removal) is complete.

Phase 2 (Tailwind removal) completed TW1-TW4 and now uses a three-slice finish:

1. **Slice TW5a: Utility signal hardening (complete).**
   - Keep app-owned utility dependencies (`sr-only`, `size-5`) removed by using app-owned semantic classes/CSS module rules at callsites.
   - Remove `@apply` from `crates/tasklens-ui/tailwind.css` so utility expansion is no longer required.
   - Tighten Tailwind extraction scope and re-baseline output.
2. **Slice TW5b: Reset-dependency canary.**
   - Identify which behaviors currently come from Tailwind preflight/reset.
   - Move only required base/reset behaviors into `crates/tasklens-ui/assets/app.css`.
   - Validate app behavior with Tailwind stylesheet disabled before deleting Tailwind files.
3. **Slice TW5c: Runtime removal and final boundary cleanup.**
   - Remove Tailwind input/output and link:
     - `crates/tasklens-ui/tailwind.css`
     - `crates/tasklens-ui/assets/tailwind.css`
     - Tailwind link in `crates/tasklens-ui/src/main.rs`
   - Restore pristine upstream `dx-components-theme.css` and keep app overrides in `assets/app.css`.
   - Run `just verify` successfully.

## Near-Term Priorities

1. Execute Slice TW5b (reset-dependency canary and base-rule ownership transfer).
2. Execute Slice TW5c (Tailwind runtime removal + final verify).

## Checklist

See `dioxus-components-migration.todo.md` for the actionable checklist.
