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
- For callsite de-Daisy cleanup slices, do not add new `assets/app.css` styles or new Tailwind CSS styles/classes unless the user explicitly approves them.

## Foundations

- [x] Create and use a pristine vendor branch for `dx components add` output, with registry URL and `rev` pinned in `dioxus-vendor-components.toml`.
- [x] Set `crates/tasklens-ui/dioxus-vendor-components.toml` `module_path = "src/dioxus_components"` (default is `src/components` when omitted from `dx` args).
- [x] Require xtask-managed vendoring to pass `--module-path src/dioxus_components` from `dioxus-vendor-components.toml` so runs cannot fall back to the default `components` module.
- [x] Split modules: keep `crates/tasklens-ui/src/dioxus_components` for vendored Dioxus Components and create `crates/tasklens-ui/src/app_components` for app UI.
- [ ] Migrate legacy Dioxus component implementations out of `crates/tasklens-ui/src/components` incrementally, one component at a time, by vendoring into `crates/tasklens-ui/src/dioxus_components` and then migrating imports/callsites.
- [x] Move app-specific components into `app_components` (examples: `task_row`, `task_editor`, `app_navbar`, `sync_indicator`, `empty_state`).
- [x] Create `assets/app.css` and link it in `main.rs`.
- [ ] Keep `crates/tasklens-ui/src/components` as a temporary compatibility shim until all imports are migrated.
- [ ] Remove Tailwind and DaisyUI build inputs (`tailwind.css`, `assets/tailwind.css`, DaisyUI plugin usage).
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

- [x] Vendor upstream `checkbox` into `crates/tasklens-ui/src/dioxus_components/checkbox` via `dioxus-vendor-components.toml` + `cargo xtask update-dioxus-components`.
- [x] Migrate Checkbox usage and behavior in:
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - `crates/tasklens-ui/src/views/task_page.rs`
- [x] Remove legacy `crates/tasklens-ui/src/components/checkbox.rs` and wire re-exports through `dioxus_components`.
- [x] Clear any remaining checkbox-related DaisyUI classes in migrated callsites.

### Chunk C (Blocked): Date Picker Vertical Slice

**Status: Blocked.** Attempted and fully reverted February 2026. Two issues must be resolved first.

Prerequisites (resolve before re-attempting):

- [ ] Fix `dx components add` registry root bug so builtin dependencies (`calendar`, `popover`) are vendored from the pinned `--rev`. Options: (a) patch `dx` CLI from `context/dioxus` submodule and rebuild, or (b) file upstream issue and wait for fix.
- [ ] Resolve WASM panic in `dioxus-primitives` caused by `time::OffsetDateTime::now_local()` defaulting in `CalendarProps`. Options: (a) enable `web` feature on `dioxus-primitives`, (b) provide explicit `today` prop at callsites, or (c) patch `context/components`.

Slice tasks (do not start until prerequisites are resolved):

- [ ] Wire upstream Date Picker wrapper and remove HTML `<input type="date">` fallback implementation.
- [ ] Migrate DatePicker usage in `crates/tasklens-ui/src/app_components/task_editor.rs` to upstream-compatible styling.
- [ ] Remove legacy date-picker-specific DaisyUI/Tailwind dropdown/menu classes.

### Chunk D: Wrapper De-Daisy (Select, Dialog, Collapsible, Calendar)

- [ ] Select: re-vendor upstream and remove DaisyUI dropdown/menu/btn assumptions.
- [ ] Dialog: re-vendor upstream and remove DaisyUI modal classes.
- [ ] Collapsible: re-vendor upstream, restore `as` support, remove Tailwind utilities.
- [ ] Calendar: re-vendor upstream and restore upstream API (`CalendarView` if needed).

### Chunk E: Replace High-Usage DaisyUI Skins With Upstream Components

- [ ] Badge: adopt upstream badge and remove `badge*` usage.
- [ ] Card: adopt upstream card and remove `card*` usage.
- [ ] Progress: adopt upstream progress and remove `progress*` usage.
- [ ] Textarea: adopt upstream textarea and remove `textarea*` usage.
- [ ] Toggle/Switch: adopt upstream toggle primitives and remove `toggle*` usage.
- [ ] Dropdown Menu + Label: adopt where needed to replace remaining `dropdown*`/`menu*` form-menu patterns.

## Completed Vertical Slices

- [x] Button: vendored to `dioxus_components`, integrated, legacy `components/button.rs` removed.
- [x] Input: vendored to `dioxus_components`, integrated, legacy `components/input.rs` removed.
- [x] Checkbox: vendored to `dioxus_components`, integrated, legacy `components/checkbox.rs` removed.

## Align Existing Dioxus Component Wrappers

- [ ] Calendar: re-vendor upstream and restore upstream API.
- [ ] Collapsible: re-vendor upstream and restore `as` support.
- [ ] Dialog: re-vendor upstream and restore upstream CSS/API.
- [ ] Select: re-vendor upstream and remove DaisyUI class overrides.

## Replace Diverged Components

- [x] Button: replace local DaisyUI `button.rs` with upstream `button` component + CSS.
- [x] Input: replace local DaisyUI `input.rs` with upstream `input` component + CSS.
- [x] Checkbox: replace local DaisyUI `checkbox.rs` with upstream `checkbox` component + CSS.
- [ ] Date Picker: wire the upstream wrapper and remove the HTML `<input type="date">` implementation.

## Verify Adopted Components

- [ ] Navbar: re-sync with upstream formatting and ensure CSS matches upstream.
- [ ] Popover: re-sync with upstream formatting and ensure CSS matches upstream.
- [ ] Slider: re-sync with upstream formatting and ensure CSS matches upstream.

## Class Debt Register (Source Of Truth For Deferred Cleanup)

- [x] `btn*` debt in app callsites is cleared (audit found no remaining `btn*` class tokens in `crates/tasklens-ui/src/app_components` and `crates/tasklens-ui/src/views`).
- [x] `input*`/`select*`/`textarea*`/`toggle*`/`join*`/`fieldset*` debt in app callsites is cleared.
- [ ] `card*`/`badge*`/`progress*` debt:
  - `crates/tasklens-ui/src/views/score_trace_page.rs`
  - `crates/tasklens-ui/src/views/balance_page.rs`
  - `crates/tasklens-ui/src/views/task_page.rs`
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/priority_task_row.rs`
  - `crates/tasklens-ui/src/app_components/empty_state.rs`
- [ ] `dropdown*`/`menu*`/`modal*` debt:
  - `crates/tasklens-ui/src/components/dialog/component.rs`
  - `crates/tasklens-ui/src/components/date_picker/component.rs`
- [ ] `loading*` debt:
  - `crates/tasklens-ui/src/app_components/loading.rs`
- [ ] DaisyUI theme utility debt (`bg-base-*`, `text-base-*`, `border-base-*`, `text-primary`, etc.) remains broadly in app components and views; burn down alongside component-skin replacement.

Audit command (run after each chunk and refresh this register):

```bash
rg -n 'class:\s*(format!\(|format_args!\(|"[^"]*\b(btn|input|select|textarea|toggle|card|badge|progress|dropdown|menu|modal|loading|fieldset|join|bg-base-|text-base-|border-base-|text-primary)\b[^"]*")' crates/tasklens-ui/src --glob '*.rs'
```

## Tailwind And DaisyUI Removal Exit Gates

- [ ] Gate 1: No DaisyUI component-skin tokens in app source (`btn*`, `input*`, `select*`, `textarea*`, `toggle*`, `card*`, `badge*`, `progress*`, `dropdown*`, `menu*`, `modal*`, `loading*`, `fieldset*`, `join*`).
- [ ] Gate 2: No DaisyUI theme utility tokens in app source (`bg-base-*`, `text-base-*`, `border-base-*`, `text-primary`, etc.).
- [ ] Gate 3: Remove Tailwind/DaisyUI inputs and outputs (`tailwind.css`, `assets/tailwind.css`, DaisyUI plugin usage, stylesheet link in `src/main.rs`).
- [ ] Gate 4: Restore pristine upstream `dx-components-theme.css`; keep app overrides in `assets/app.css`.
- [ ] Gate 5: Run `just verify` successfully after Tailwind/DaisyUI removal.

## Tracking

- [ ] Keep `dioxus-components-migration.md` inventory and divergence summary up to date as components are migrated.
