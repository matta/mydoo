# Dioxus Components Migration Checklist (Active)

## Tracking Policy

- Track this migration in this checklist and `dioxus-components-migration.md`.
- Do not create or update `bd` items for routine migration slices unless the user explicitly asks for `bd` tracking.
- When slice status changes, update this file directly in the same change.

## Working Agreement (Prevent Slice Drift)

- Every active slice must include both:
  - component adoption work (vendor/integrate/remove legacy), and
  - callsite class cleanup work (or explicit deferred file list in this checklist).
- The active chunk must point to the highest-impact remaining debt, not only the next component name.
- Keep migration inventory/status in `dioxus-components-migration.md` synced in the same change as checklist updates.
- Do not introduce new DaisyUI classes.
- For app-owned callsite utility replacements, prefer component-scoped CSS module classes over adding new shared utility helpers in `crates/tasklens-ui/assets/app.css`.

## Completed Work (Condensed)

- Foundations complete: vendor branch workflow, pinned registry config, `module_path = "src/dioxus_components"`, `app_components` split, and `assets/app.css` wiring.
- Phase 1 complete: DaisyUI plugin removed and DaisyUI class/theme debt cleared (exit gates 1-4).
- Completed slices include: Button, Input, Checkbox, Select, Dialog, Collapsible, Badge, Card, Progress, Textarea, Toggle/Switch, Label, Slider, and app-owned Date/DateTime wrappers with CSS modules.
- CSS modules feasibility gate is complete for app-owned components.

For full historical detail on removed checklist items, inspect git history:

```bash
git log -- docs/design/dioxus-components-migration.todo.md
git show <commit>:docs/design/dioxus-components-migration.todo.md
```

## Tailwind Audit Snapshot (2026-02-15)

- Generated `crates/tasklens-ui/assets/tailwind.css` is currently `395` lines (`wc -l`) after TW5a scope narrowing.
- Tailwind preflight/reset and properties layers are still present due to `@import "tailwindcss"` in `crates/tasklens-ui/tailwind.css`.
- Remaining app-owned Rust utility dependencies from the TW5a checklist (`sr-only`, `size-5`) are removed from app-owned callsites.
- `@apply` has been removed from `crates/tasklens-ui/tailwind.css`; Tailwind runtime dependency is now driven by reset/theme/base decisions.
- Compiled output includes likely extraction noise selectors (for example `.container` and `.table`) without matching app-owned Rust callsites.

## Current Critical Path (Execute Top To Bottom)

- [x] Slice TW1: finalize compatibility shim removal (precondition for Tailwind runtime removal).
  - [x] Migrate callsites off `crate::components::{dialog, navbar, popover, Alert}`.
  - [x] Navbar: re-sync with upstream formatting and CSS.
  - [x] Popover: re-sync with upstream formatting and CSS.
  - [x] Replace `Alert` usage with app-owned styling (no `alert-*` Daisy/Tailwind tokens).
  - [x] Remove now-unused modules from `crates/tasklens-ui/src/components`. (Note: shim modules still exist but callsites migrated).
  - [x] Keep vendored `dioxus_components` on upstream `style.css` + `document::Link` patterns (no CSS-module conversion in vendored code).
- [x] Slice TW2: de-tailwind Task Editor surfaces and migrate to CSS modules.
  - [x] `crates/tasklens-ui/src/app_components/task_editor.rs`
  - [x] `crates/tasklens-ui/src/app_components/move_picker.rs`
  - [x] `crates/tasklens-ui/src/views/auth.rs`
- [x] Slice TW3: de-tailwind task-flow support surfaces.
  - [x] `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
  - [x] `crates/tasklens-ui/src/app_components/task_row.rs`
  - [x] `crates/tasklens-ui/src/app_components/search_panel.rs`
  - [x] `crates/tasklens-ui/src/app_components/sync_indicator.rs`
  - [x] `crates/tasklens-ui/src/app_components/load_error_view.rs`
  - [x] `crates/tasklens-ui/src/app_components/task_input.rs`
  - [x] `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - [x] `crates/tasklens-ui/src/app_components/page_header.rs`
  - [x] `crates/tasklens-ui/src/app_components/back_button.rs`
- [x] Slice TW4: de-tailwind page shells and analytics surfaces.
  - [x] `crates/tasklens-ui/src/views/plan_page.rs`
  - [x] `crates/tasklens-ui/src/views/do_page.rs`
  - [x] `crates/tasklens-ui/src/views/task_page.rs`
  - [x] `crates/tasklens-ui/src/views/balance_page.rs`
  - [x] `crates/tasklens-ui/src/views/score_trace_page.rs`
- [x] Slice TW5a: harden utility signal before runtime removal.
  - [x] Replace remaining utility-class dependencies in app-owned Rust callsites (`sr-only`, `size-5`) with app-owned semantic classes/CSS modules at point of use.
  - [x] Remove `@apply` usage from `crates/tasklens-ui/tailwind.css` and use explicit CSS declarations.
  - [x] Tighten Tailwind extraction scope (`@source`) to reduce false-positive utility generation and re-baseline output size.
  - [x] Update this checklist and the active plan doc with the new baseline after re-generation.
- [ ] Slice TW5b: run reset-dependency canary and move required base behavior into app CSS.
  - [ ] Inventory app behavior currently supplied by Tailwind preflight/reset (form defaults, heading/list defaults, media defaults, hidden behavior).
  - [ ] Add only required base/reset rules to `crates/tasklens-ui/assets/app.css`.
  - [ ] Validate UI behavior with Tailwind stylesheet temporarily disabled before deleting Tailwind files.
- [ ] Slice TW5c: remove Tailwind runtime and close Phase 2.
  - [ ] Interim signal: generated `crates/tasklens-ui/assets/tailwind.css` shows no app-owned utility selectors and is dominated by intentional base/reset content (baseline is 395 lines on 2026-02-15 after TW5a).
  - [ ] Remove `crates/tasklens-ui/tailwind.css`.
  - [ ] Remove `crates/tasklens-ui/assets/tailwind.css`.
  - [ ] Remove Tailwind stylesheet link from `crates/tasklens-ui/src/main.rs`.
  - [ ] Restore pristine upstream `crates/tasklens-ui/assets/dx-components-theme.css`.
  - [ ] Keep app overrides in `crates/tasklens-ui/assets/app.css`.
  - [ ] `just verify` passes.

## Two-Phase DaisyUI Then Tailwind Exit Gates

Phase 1 (DaisyUI removal): complete.

- [x] Gate 1: No DaisyUI component-skin tokens in app source.
- [x] Gate 2: No DaisyUI theme utility tokens in app source.
- [x] Gate 3: DaisyUI plugin usage removed from Tailwind configuration.
- [x] Gate 4: `just verify` passed after DaisyUI removal.

Phase 2 (Tailwind removal): remaining.

- [x] Gate 5: No app-owned Rust callsites depend on Tailwind utility selectors (`sr-only`, `size-5`, and similar).
- [x] Gate 6: No Tailwind-only directives remain in app-owned Tailwind input (`@apply` removed; runtime dependency isolated to explicit reset decision).
- [ ] Gate 7: Reset-dependency canary passes after moving required base behavior into `assets/app.css`.
- [ ] Gate 8: Remove Tailwind input/output and stylesheet link, restore pristine upstream `dx-components-theme.css`, keep app overrides in `assets/app.css`, and run `just verify`.

## Audit Commands

```bash
rg -n 'class:\s*(format!\(|format_args!\(|"[^"]*\b(btn|input|select|textarea|toggle|card|badge|progress|dropdown|menu|modal|loading|fieldset|join|bg-base-|text-base-|border-base-|text-primary)\b[^"]*")' crates/tasklens-ui/src --glob '*.rs'

# Phase 2 utility-class debt inventory (app-owned callsites)
rg -n 'class:\s*"[^"]*\b(container|mx-auto|max-w-|min-h-screen|flex|grid|gap-|space-|p[trblxy]?-[0-9]|m[trblxy]?-[0-9]|w-|h-|text-(xs|sm|base|lg|2xl)|font-(bold|medium|semibold)|rounded|border|shadow|overflow-|items-|justify-|cursor-|transition-|opacity-|hover:|absolute|relative|z-|top-|left-|right-|bottom-)\b[^"]*"' crates/tasklens-ui/src/{app_components,views,main.rs} --glob '*.rs'

# Phase 2 residual utility dependencies in app-owned Rust callsites
rg -n 'class:\s*"[^"]*\b(sr-only|size-5)\b[^"]*"' crates/tasklens-ui/src/{app_components,views,main.rs} --glob '*.rs'

# Tailwind directives that still create runtime dependency
rg -n '@apply|@utility|@source|@theme' crates/tasklens-ui/tailwind.css

# Tailwind output health check (user-visible Phase 2 signal)
wc -l crates/tasklens-ui/assets/tailwind.css
rg -n '^\s*\.(container|table|relative|static|block|flex|hidden|inline|transform|border|line-through|outline|filter|transition)\b' crates/tasklens-ui/assets/tailwind.css
rg -n '\.(container|max-w-2xl|grid-cols-2|bg-app-surface|border-app-border|text-xs|font-bold|rounded-md)\b' crates/tasklens-ui/assets/tailwind.css
```
