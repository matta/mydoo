# Dioxus Components Migration Checklist

## Tracking Policy

- Track this migration in this checklist and `dioxus-components-migration.md`.
- Do not create or update `bd` items for routine migration slices unless the user explicitly asks for `bd` tracking.
- When slice status changes, update this file directly in the same change.

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

## Active Chunk: Button Vertical Slice

- [x] Vendor upstream `button` into `crates/tasklens-ui/src/dioxus_components/button` via `dioxus-vendor-components.toml` + `cargo xtask update-dioxus-components`.
- [x] Integrate vendored `dioxus_components::button` into app code using direct callsite migration to `crate::dioxus_components::button::{Button, ButtonVariant}`.
- [x] Remove legacy `crates/tasklens-ui/src/components/button.rs` once no callsites depend on it.
- [x] Update module declarations/re-exports so Button resolves from `dioxus_components` rather than legacy `components`.
- [x] Update migration inventory/divergence notes after Button lands.

## Deferred After Button Slice: DaisyUI Button Markup Cleanup

- [ ] Replace raw RSX `button` elements that still use DaisyUI `btn*` classes with Dioxus Button usage or app-owned CSS in:
  - `crates/tasklens-ui/src/views/plan_page.rs`
  - `crates/tasklens-ui/src/app_components/task_row.rs`
  - `crates/tasklens-ui/src/app_components/sync_indicator.rs`
  - `crates/tasklens-ui/src/app_components/search_panel.rs`
  - `crates/tasklens-ui/src/app_components/app_navbar.rs`
- [ ] Remove remaining `btn*` styling assumptions passed to `Button { class: ... }` where they rely on DaisyUI tokens (for example: `btn-sm`, `btn-circle`, `btn-xs`) and replace with upstream-compatible/app CSS classes.
- [ ] Replace non-button element DaisyUI `btn*` classes used for list-option affordances (for example in `move_picker` and `components/select/component.rs`) with upstream-compatible patterns.

## Deferred After Input Slice: DaisyUI And Tailwind Input Callsite Cleanup

- [ ] Remove DaisyUI/Tailwind-oriented input class tokens from migrated `Input` callsites and replace with upstream-compatible/app-owned classes in:
  - `crates/tasklens-ui/src/app_components/task_input.rs`
  - `crates/tasklens-ui/src/app_components/task_editor.rs`
  - `crates/tasklens-ui/src/app_components/doc_id_manager.rs`
- [ ] Ensure migrated Input callsites do not pass component-skin classes that shadow vendored defaults unless explicitly intended and documented.

## Align Existing Dioxus Component Wrappers

- [ ] Calendar: re-vendor upstream and restore upstream API (remove extra wrapper, restore CalendarView if needed).
- [ ] Collapsible: re-vendor upstream, restore `as` support, remove Tailwind classes.
- [ ] Dialog: re-vendor upstream, remove DaisyUI modal classes, restore upstream CSS.
- [ ] Select: re-vendor upstream and remove DaisyUI class overrides.

## Replace Diverged Components

- [x] Button: replace local DaisyUI `button.rs` with upstream `button` component + CSS.
- [ ] Checkbox: replace local DaisyUI `checkbox.rs` with upstream `checkbox` component + CSS.
- [ ] Input: replace local DaisyUI `input.rs` with upstream `input` component + CSS.
- [ ] Date Picker: wire the upstream wrapper and remove the HTML `<input type="date">` implementation.

## Verify Adopted Components

- [ ] Navbar: re-sync with upstream formatting and ensure CSS matches upstream.
- [ ] Popover: re-sync with upstream formatting and ensure CSS matches upstream.
- [ ] Slider: re-sync with upstream formatting and ensure CSS matches upstream.

## Adopt Missing Upstream Components (Decide And Implement)

- [ ] Accordion: decide adopt or defer; if adopt, vendor upstream component and replace any ad-hoc collapse UI.
- [ ] Alert Dialog: decide adopt or defer; if adopt, replace destructive confirmation dialogs.
- [ ] Aspect Ratio: decide adopt or defer; if adopt, replace fixed-ratio media styles.
- [ ] Avatar: decide adopt or defer; if adopt, replace custom avatar UI.
- [ ] Badge: decide adopt or defer; if adopt, replace DaisyUI badges.
- [ ] Card: decide adopt or defer; if adopt, replace DaisyUI cards.
- [ ] Context Menu: decide adopt or defer; if adopt, replace context menu patterns.
- [ ] Dropdown Menu: decide adopt or defer; if adopt, replace DaisyUI dropdown usage.
- [ ] Form: decide adopt or defer; if adopt, standardize form layouts.
- [ ] Hover Card: decide adopt or defer; if adopt, replace hover preview patterns.
- [ ] Label: decide adopt or defer; if adopt, replace DaisyUI labels.
- [ ] Menubar: decide adopt or defer; if adopt, standardize menu bars.
- [ ] Progress: decide adopt or defer; if adopt, replace DaisyUI progress bars.
- [ ] Radio Group: decide adopt or defer; if adopt, replace radio inputs.
- [ ] Scroll Area: decide adopt or defer; if adopt, replace custom scroll containers.
- [ ] Separator: decide adopt or defer; if adopt, replace separators in menus and cards.
- [ ] Sheet: decide adopt or defer; if adopt, replace slide-over panels.
- [ ] Sidebar: decide adopt or defer; if adopt, replace sidebar navigation.
- [ ] Skeleton: decide adopt or defer; if adopt, replace loading placeholders.
- [ ] Switch: decide adopt or defer; if adopt, replace DaisyUI toggles.
- [ ] Tabs: decide adopt or defer; if adopt, replace tab UI.
- [ ] Textarea: decide adopt or defer; if adopt, replace DaisyUI textareas.
- [ ] Toast: decide adopt or defer; if adopt, replace notification patterns.
- [ ] Toggle: decide adopt or defer; if adopt, replace DaisyUI toggle buttons.
- [ ] Toggle Group: decide adopt or defer; if adopt, replace grouped toggle UI.
- [ ] Toolbar: decide adopt or defer; if adopt, standardize toolbar layouts.
- [ ] Tooltip: decide adopt or defer; if adopt, replace hover hints.

## Tailwind And DaisyUI Removal By Category

- [ ] Layout and spacing: replace utility classes in `views/*` with `app.css` layout classes.
- [ ] Typography: replace `text-*` and `font-*` utilities with `app.css` typography rules.
- [ ] Color and theme: replace `bg-base-*` and `text-base-*` classes with CSS variables.
- [ ] Interactive states: move hover/focus styling into component CSS or `app.css`.
- [ ] Component skins: replace DaisyUI class usage with Dioxus components.

## Tracking

- [ ] Keep `dioxus-components-migration.md` inventory and divergence summary up to date as components are migrated.
