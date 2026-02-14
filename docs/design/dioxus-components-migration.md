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
- Tailwind utility usage matches in app-owned source: reduced to ~46 (from ~115 baseline).
- Highest remaining hotspots: `score_trace_page.rs` (20), `balance_page.rs` (12), `task_page.rs` (6).

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
- Keep `dx-components-theme.css` pristine upstream.
- Put app-specific layout/overrides in `assets/app.css`.

## Remaining Exit Criteria

Phase 1 (DaisyUI removal) is complete.

Phase 2 (Tailwind removal) is split into reviewable slices:

1. **Slice TW1: Compatibility shim finalization (precondition).**
   - Migrate remaining imports from `crate::components::{dialog, navbar, popover, Alert}`.
   - Re-sync Navbar and Popover to upstream formatting/CSS parity.
   - Remove obsolete `src/components` files as migrations land.
2. **Slice TW2: Task editor surface de-tailwind.**
   - Target `task_editor.rs` first (largest utility-class hotspot), plus related modal/move-picker files.
   - Replace Tailwind utility strings with app-owned semantic classes.
3. **Slice TW3: Task-flow support surfaces de-tailwind.**
   - Migrate `doc_id_manager.rs`, `task_row.rs`, `search_panel.rs`, `sync_indicator.rs`, `load_error_view.rs`, and related small app components.
4. **Slice TW4: Page shell and analytics surface de-tailwind.**
   - Migrate `plan_page.rs`, `do_page.rs`, `task_page.rs`, `balance_page.rs`, `score_trace_page.rs`.
5. **Slice TW5: Tailwind runtime removal and final CSS boundary cleanup.**
   - Interim signal before deletion: generated `assets/tailwind.css` should be near-empty (no app utility selectors from the audit set and materially smaller than the current ~1196-line baseline).
   - Remove Tailwind input/output and link:
     - `crates/tasklens-ui/tailwind.css`
     - `crates/tasklens-ui/assets/tailwind.css`
     - Tailwind link in `crates/tasklens-ui/src/main.rs`
   - Restore pristine upstream `dx-components-theme.css` and keep app overrides in `assets/app.css`.
   - Run `just verify` successfully.

## Near-Term Priorities

1. Execute Slice TW1 (`src/components` shim removal and Navbar/Popover parity).
2. Execute Slice TW2 (Task Editor hotspot cleanup).
3. Execute Slice TW3 (task-flow support surfaces).
4. Execute Slice TW4 (page-shell/analytics surfaces).
5. Execute Slice TW5 (Tailwind runtime removal + final verify).

## Checklist

See `dioxus-components-migration.todo.md` for the actionable checklist.
