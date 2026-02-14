# Dioxus Components Migration Checklist

## Tracking Policy

- Track this migration in this checklist and `dioxus-components-migration.md`.
- Do not create or update `bd` items for routine migration slices unless the user explicitly asks for `bd` tracking.
- When slice status changes, update this file directly in the same change.

## Working Agreement (Prevent Slice Drift)

- Every component slice must include both:
  - component adoption work (vendor/integrate/remove legacy), and
  - callsite class cleanup work (or explicit deferred file list in this checklist).
- The active chunk must always point to the highest-impact remaining class debt, not only the next component name.
- `Replace Diverged Components` status and migration inventory status must be updated in the same commit as slice changes.
- For callsite de-Daisy cleanup slices (Phase 1), do not add new `assets/app.css` styles unless explicitly approved by the user; adding Tailwind CSS classes is acceptable when it helps replace DaisyUI classes in focused diffs and keeps remaining Tailwind debt explicit.

## Foundations

- [x] Create and use a pristine vendor branch for `dx components add` output, with registry URL and `rev` pinned in `dioxus-vendor-components.toml`.
- [x] Set `crates/tasklens-ui/dioxus-vendor-components.toml` `module_path = "src/dioxus_components"` (default is `src/components` when omitted from `dx` args).
- [x] Require xtask-managed vendoring to pass `--module-path src/dioxus_components` from `dioxus-vendor-components.toml` so runs cannot fall back to the default `components` module.
- [x] Split modules: keep `crates/tasklens-ui/src/dioxus_components` for vendored Dioxus Components and create `crates/tasklens-ui/src/app_components` for app UI.
- [ ] Migrate legacy Dioxus component implementations out of `crates/tasklens-ui/src/components` incrementally, one component at a time, by vendoring into `crates/tasklens-ui/src/dioxus_components` and then migrating imports/callsites.
- [x] Move app-specific components into `app_components` (examples: `task_row`, `task_editor`, `app_navbar`, `sync_indicator`, `empty_state`).
- [x] Create `assets/app.css` and link it in `main.rs`.
- [ ] Keep `crates/tasklens-ui/src/components` as a temporary compatibility shim until all imports are migrated.
- [x] Phase 1: Remove DaisyUI plugin usage from Tailwind configuration.
- [ ] Phase 2: Remove Tailwind build inputs (`tailwind.css`, `assets/tailwind.css`) and stylesheet link in `src/main.rs`.
- [ ] Replace local edits in `dx-components-theme.css` with a pristine upstream copy; move overrides to `app.css`.

## Current Critical Path (Execute Top To Bottom)

### Chunk A (Completed): Button And Input Callsite De-Daisy

- [x] Remove remaining `btn*` classes from raw RSX buttons in:
  - `crates/tasklens-ui/src/views/plan_page.rs`
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/sync_indicator.rs`
  - `crates/tasklens-ui/src/app_components/search_panel.rs`
  - `crates/tasklens-ui/src/app_components/app_navbar.rs`
- [x] Remove DaisyUI-flavored Button class tokens (`btn-sm`, `btn-xs`, `btn-circle`, `btn-square`, `btn-primary`) and replace with app-owned/upstream-compatible classes in:
  - `crates/tasklens-ui/src/app_components/task_editor.rs`
  - `crates/tasklens-ui/src/views/do_page.rs`
  - any additional Button callsites found by audit.
- [x] Replace non-button `btn*` list affordances with upstream-compatible patterns in:
  - `crates/tasklens-ui/src/app_components/move_picker.rs`
  - `crates/tasklens-ui/src/components/select/component.rs`
- [x] Remove DaisyUI/Tailwind-oriented input/select/textarea/toggle/join/fieldset classes from Input-adjacent callsites in:
  - `crates/tasklens-ui/src/app_components/task_input.rs`
  - `crates/tasklens-ui/src/app_components/task_editor.rs`
  - `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
  - `crates/tasklens-ui/src/app_components/search_panel.rs`
  - `crates/tasklens-ui/src/app_components/sync_indicator.rs`
- [x] After completing Chunk A, refresh the Class Debt Register section below with remaining file hotspots.

### Chunk B (Completed): Checkbox Vertical Slice

- [x] Vendor upstream `checkbox` into `crates/tasklens-ui/src/dioxus_components/checkbox` via `dioxus-vendor-components.toml` + `cargo xtask dx-components vendor`.
- [x] Migrate Checkbox usage and behavior in:
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - `crates/tasklens-ui/src/views/task_page.rs`
- [x] Remove legacy `crates/tasklens-ui/src/components/checkbox.rs` and wire re-exports through `dioxus_components`.
- [x] Clear any remaining checkbox-related DaisyUI classes in migrated callsites.

### Chunk C (Completed): Native Date/Datetime Vertical Slice + CSS Modules Pilot

**Status: Completed February 2026.** Browser-native date/datetime controls are the short-term direction while de-Daisy work continues.

- [x] Replace legacy `DatePicker` wrapper (DaisyUI/Tailwind classes) with app-owned `DateInput` and `DateTimeInput` in `crates/tasklens-ui/src/app_components/date_time_inputs/` using CSS modules.
- [x] Migrate callsite in `crates/tasklens-ui/src/app_components/task_editor.rs` from `DatePicker` to `DateInput`.
- [x] Remove legacy `crates/tasklens-ui/src/components/date_picker/` directory (both `mod.rs` and `component.rs`).
- [x] Pilot Dioxus `#[css_module(...)]` on app-owned date/datetime wrappers — verified production build produces hashed, scoped class names.
- [x] Add or refresh tests covering native date input behavior and value conversion.

### Chunk D (Completed): Wrapper De-Daisy (Select, Dialog, Collapsible)

- [x] Select: re-vendor upstream and remove DaisyUI dropdown/menu/btn assumptions.
- [x] Dialog: re-vendor upstream and remove DaisyUI modal classes.
- [x] Collapsible: re-vendor upstream, restore `as` support, remove Tailwind utilities.

### Chunk E: Replace High-Usage DaisyUI Skins With Upstream Components (In Progress)

- [x] Badge: adopt upstream badge and remove `badge*` usage.
- [x] Card: adopt upstream card and remove `card*` usage in:
  - `crates/tasklens-ui/src/views/balance_page.rs`
  - `crates/tasklens-ui/src/views/task_page.rs`
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - `crates/tasklens-ui/src/app_components/empty_state.rs`
  - `crates/tasklens-ui/src/views/score_trace_page.rs`
- [x] Progress: adopt upstream progress and remove `progress*` usage.
- [x] Textarea: adopt upstream textarea and remove `textarea*` usage.
- [x] Toggle/Switch: adopt upstream toggle primitives and remove `toggle*` usage.
- [x] Dropdown Menu + Label: adopt where needed to replace remaining `dropdown*`/`menu*` form-menu patterns.
- [x] Slider: adopt upstream slider and remove `range*` usage.

## Completed Vertical Slices

- [x] Button: vendored to `dioxus_components`, integrated, legacy `components/button.rs` removed.
- [x] Input: vendored to `dioxus_components`, integrated, legacy `components/input.rs` removed.
- [x] Checkbox: vendored to `dioxus_components`, integrated, legacy `components/checkbox.rs` removed.
- [x] Badge: vendored to `dioxus_components`, integrated.
- [x] Card: vendored to `dioxus_components`, integrated.
- [x] Progress: vendored to `dioxus_components`, integrated.
- [x] Date/Datetime Inputs: app-owned `DateInput` and `DateTimeInput` with CSS modules, legacy `components/date_picker/` removed.
- [x] Select: vendored to `dioxus_components`, integrated, legacy `components/select/` removed.
- [x] Slider: vendored to `dioxus_components`, integrated, legacy `components/slider/` removed.
- [x] Label: vendored to `dioxus_components`, integrated.

## Cross-Cutting Gate: CSS Modules Feasibility

- [x] Pilot Dioxus `#[css_module(...)]` on one app-owned component (date/datetime wrappers in Chunk C).
- [x] Verify production build behavior for the pilot (asset output produces hashed, scoped, minified CSS).
- [x] Verify test stability for the pilot flow (unit/E2E coverage for the touched date path).
- [x] Only after pilot success: update `dioxus-components-migration.md` guidance to recommend CSS modules for app-owned components.
- [ ] Keep vendored `dioxus_components` on upstream `style.css` + `document::Link` regardless of pilot outcome.

## Align Existing Dioxus Component Wrappers

- [x] Collapsible: re-vendor upstream and restore `as` support.
- [x] Dialog: re-vendor upstream and restore upstream CSS/API.
- [x] Select: re-vendor upstream and remove DaisyUI class overrides.

## Replace Diverged Components

- [x] Button: replace local DaisyUI `button.rs` with upstream `button` component + CSS.
- [x] Input: replace local DaisyUI `input.rs` with upstream `input` component + CSS.
- [x] Checkbox: replace local DaisyUI `checkbox.rs` with upstream `checkbox` component + CSS.
- [x] Date/Datetime Inputs: replaced with app-owned CSS-module-scoped wrappers.
- [x] Select: replaced local `select/mod.rs` with upstream `select` component + CSS.

## Verify Adopted Components

- [ ] Navbar: re-sync with upstream formatting and ensure CSS matches upstream.
- [ ] Popover: re-sync with upstream formatting and ensure CSS matches upstream.
- [x] Slider: re-vendored to `dioxus_components`, integrated, legacy implementation removed.

## Class Debt Register (Source Of Truth For Deferred Cleanup)

- [x] `btn*` debt in app callsites is cleared (audit found no remaining `btn*` class tokens in `crates/tasklens-ui/src/app_components` and `crates/tasklens-ui/src/views`).
- [x] `input*`/`select*`/`textarea*`/`toggle*`/`join*`/`fieldset*` debt in app callsites is cleared.
- [x] `badge*` debt in app callsites is cleared.
- [x] `card*` debt in app callsites is cleared.
- [x] `progress*` debt in app callsites is cleared.
- [x] `range*` debt: cleared.
- [x] `dropdown*`/`menu*`/`modal*` debt:
  - `crates/tasklens-ui/src/dioxus_components/dialog/component.rs` (uses upstream `dialog-backdrop` and `dialog`)
- [x] Native date/datetime wrapper utility-class debt: cleared (legacy `date_picker` removed, new wrappers use CSS modules).
- [x] `loading*` debt: cleared (replaced with CSS-module-scoped spinner in `loading.rs`).
- [x] DaisyUI theme utility debt (`bg-base-*`, `text-base-content`, `border-base-*`, `text-primary`) cleared from app components and views. Replaced with Tailwind `@theme` bridge colors (`app-surface`, `app-text`, `app-border`, `app-accent`) that reference upstream CSS variables.

Audit command (run after each chunk and refresh this register):

```bash
rg -n 'class:\s*(format!\(|format_args!\(|"[^"]*\b(btn|input|select|textarea|toggle|card|badge|progress|dropdown|menu|modal|loading|fieldset|join|bg-base-|text-base-|border-base-|text-primary)\b[^"]*")' crates/tasklens-ui/src --glob '*.rs'
```

## Two-Phase DaisyUI Then Tailwind Exit Gates

Phase 1 (DaisyUI removal):

- [x] Gate 1: No DaisyUI component-skin tokens in app source (`btn*`, `input*`, `select*`, `textarea*`, `toggle*`, `card*`, `badge*`, `progress*`, `dropdown*`, `menu*`, `modal*`, `loading*`, `fieldset*`, `join*`).
- [x] Gate 2: No DaisyUI theme utility tokens in app source (`bg-base-*`, `text-base-*`, `border-base-*`, `text-primary`, etc.).
- [x] Gate 3: DaisyUI plugin usage removed from Tailwind configuration.
- [x] Gate 4: Run `just verify` successfully after DaisyUI removal (Tailwind runtime may remain temporarily).

Phase 2 (Tailwind removal):

- [ ] Gate 5: Remove Tailwind inputs/outputs (`tailwind.css`, `assets/tailwind.css`, stylesheet link in `src/main.rs`).
- [ ] Gate 6: Restore pristine upstream `dx-components-theme.css`; keep app overrides in `assets/app.css`.
- [ ] Gate 7: Run `just verify` successfully after Tailwind removal.

## Tracking

- [x] Keep `dioxus-components-migration.md` inventory and divergence summary up to date as components are migrated.
