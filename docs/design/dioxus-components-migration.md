# Dioxus Components Migration: Gap Analysis And Plan

## Contents

- [Goals](#goals)
- [Current State](#current-state)
- [Upstream Architecture](#upstream-architecture)
- [Component Inventory And Divergence](#component-inventory-and-divergence)
- [Divergence Summary](#divergence-summary)
- [Tailwind And DaisyUI Usage Audit](#tailwind-and-daisyui-usage-audit)
- [Gap Analysis Against Dioxus Components](#gap-analysis-against-dioxus-components)
- [Module Boundary And Naming](#module-boundary-and-naming)
- [Dependency Strategy For Dioxus Primitives](#dependency-strategy-for-dioxus-primitives)
- [CSS Strategy De Tailwind](#css-strategy-de-tailwind)
- [Build And Asset Pipeline Changes](#build-and-asset-pipeline-changes)
- [DX Installation Processing Implications](#dx-installation-processing-implications)
- [Upstream Tracking Strategy](#upstream-tracking-strategy)
- [Component Acquisition Workflow](#component-acquisition-workflow)
- [Representative Diffs](#representative-diffs)
- [Migration Plan And Priorities](#migration-plan-and-priorities)
- [Checklist](#checklist)

## Goals

- Use Dioxus Labs components more directly with minimal local edits.
- Match the upstream component styling as closely as possible.
- Preserve upstream history and enable clean merges while keeping local patches.
- Remove Tailwind and DaisyUI and replace them with upstream CSS plus minimal app CSS.

## Current State

- The app now has `app_components` for app-specific UI, with `components` retained as a temporary compatibility shim for legacy imports.
- Several components already wrap `dioxus-primitives` (Dialog, Collapsible, Calendar, Popover, Select, Slider), but with local styling and API drift.
- Tailwind and DaisyUI classes are used broadly across components and views.
- Tailwind is still loaded globally while migration work continues; `app.css` is now linked for additive app-level styling.
- `dioxus-primitives` is a git dependency, not tied to a vendored upstream snapshot.
- A local `dx-components-theme.css` is already loaded globally, but it diverges from upstream.

## Upstream Architecture

Upstream reference: [DioxusLabs/components](https://github.com/DioxusLabs/components)

- `dioxus-primitives` provides logic, accessibility, and state management.
- Styled components live in `preview/src/components/*` and use plain per-component CSS (`style.css`) plus CSS variables from the theme.
- Each styled component typically links its CSS via `document::Link { rel: "stylesheet", href: asset!("./style.css") }`.
- Global theming is provided by `preview/assets/dx-components-theme.css` using CSS variables and a shared font.

## Component Inventory And Divergence

Status legend:

- Adopted: aligned with upstream, minor formatting-only diffs.
- Wrapper/Modified: uses primitives but diverges in API or styling.
- Diverged: app-specific replacement that does not follow upstream.
- Missing: no local equivalent.

| Component     | Upstream Path                                                                                                                   | Local Path                                             | Status                                                           | Recommended Action                                         |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------- | ---------------------------------------------------------- |
| Accordion     | [preview/src/components/accordion](https://github.com/DioxusLabs/components/tree/main/preview/src/components/accordion)         | —                                                      | Missing                                                          | Adopt if needed (likely for collapsible content in views). |
| Alert Dialog  | [preview/src/components/alert_dialog](https://github.com/DioxusLabs/components/tree/main/preview/src/components/alert_dialog)   | —                                                      | Missing                                                          | Adopt for destructive confirmations and modal alerts.      |
| Aspect Ratio  | [preview/src/components/aspect_ratio](https://github.com/DioxusLabs/components/tree/main/preview/src/components/aspect_ratio)   | —                                                      | Missing                                                          | Adopt if media thumbnails need consistent sizing.          |
| Avatar        | [preview/src/components/avatar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/avatar)               | —                                                      | Missing                                                          | Adopt if user/profile UI is needed.                        |
| Badge         | [preview/src/components/badge](https://github.com/DioxusLabs/components/tree/main/preview/src/components/badge)                 | —                                                      | Missing                                                          | Adopt to replace DaisyUI `badge` usage.                    |
| Button        | [preview/src/components/button](https://github.com/DioxusLabs/components/tree/main/preview/src/components/button)               | `crates/tasklens-ui/src/components/button.rs`          | Diverged (DaisyUI classes)                                       | Replace with upstream component and CSS.                   |
| Calendar      | [preview/src/components/calendar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/calendar)           | `crates/tasklens-ui/src/components/calendar/`          | Wrapper/Modified (extra wrapper, missing CalendarView)           | Re-vendor and restore upstream API.                        |
| Card          | [preview/src/components/card](https://github.com/DioxusLabs/components/tree/main/preview/src/components/card)                   | —                                                      | Missing                                                          | Adopt to replace DaisyUI `card` usage.                     |
| Checkbox      | [preview/src/components/checkbox](https://github.com/DioxusLabs/components/tree/main/preview/src/components/checkbox)           | `crates/tasklens-ui/src/components/checkbox.rs`        | Diverged (DaisyUI classes)                                       | Replace with upstream component and CSS.                   |
| Collapsible   | [preview/src/components/collapsible](https://github.com/DioxusLabs/components/tree/main/preview/src/components/collapsible)     | `crates/tasklens-ui/src/components/collapsible/`       | Wrapper/Modified (missing `as` support, Tailwind classes)        | Re-vendor and restore upstream API.                        |
| Context Menu  | [preview/src/components/context_menu](https://github.com/DioxusLabs/components/tree/main/preview/src/components/context_menu)   | —                                                      | Missing                                                          | Adopt if context menus are needed.                         |
| Date Picker   | [preview/src/components/date_picker](https://github.com/DioxusLabs/components/tree/main/preview/src/components/date_picker)     | `crates/tasklens-ui/src/components/date_picker/mod.rs` | Diverged (simple HTML input; upstream wrapper exists but unused) | Replace with upstream component and wire `component.rs`.   |
| Dialog        | [preview/src/components/dialog](https://github.com/DioxusLabs/components/tree/main/preview/src/components/dialog)               | `crates/tasklens-ui/src/components/dialog/`            | Wrapper/Modified (DaisyUI classes, no upstream CSS)              | Re-vendor and restore upstream styling.                    |
| Dropdown Menu | [preview/src/components/dropdown_menu](https://github.com/DioxusLabs/components/tree/main/preview/src/components/dropdown_menu) | —                                                      | Missing                                                          | Adopt to replace DaisyUI `dropdown` usage.                 |
| Form          | [preview/src/components/form](https://github.com/DioxusLabs/components/tree/main/preview/src/components/form)                   | —                                                      | Missing                                                          | Adopt if form layouts need standardization.                |
| Hover Card    | [preview/src/components/hover_card](https://github.com/DioxusLabs/components/tree/main/preview/src/components/hover_card)       | —                                                      | Missing                                                          | Adopt if hover previews are needed.                        |
| Input         | [preview/src/components/input](https://github.com/DioxusLabs/components/tree/main/preview/src/components/input)                 | `crates/tasklens-ui/src/components/input.rs`           | Diverged (DaisyUI classes, reduced API surface)                  | Replace with upstream component and CSS.                   |
| Label         | [preview/src/components/label](https://github.com/DioxusLabs/components/tree/main/preview/src/components/label)                 | —                                                      | Missing                                                          | Adopt to replace DaisyUI `label` usage.                    |
| Menubar       | [preview/src/components/menubar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/menubar)             | —                                                      | Missing                                                          | Adopt if menu bars are needed.                             |
| Navbar        | [preview/src/components/navbar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/navbar)               | `crates/tasklens-ui/src/components/navbar/`            | Adopted (minor formatting diffs)                                 | Keep and re-sync to upstream formatting.                   |
| Popover       | [preview/src/components/popover](https://github.com/DioxusLabs/components/tree/main/preview/src/components/popover)             | `crates/tasklens-ui/src/components/popover/`           | Adopted (minor formatting diffs)                                 | Keep and re-sync to upstream formatting.                   |
| Progress      | [preview/src/components/progress](https://github.com/DioxusLabs/components/tree/main/preview/src/components/progress)           | —                                                      | Missing                                                          | Adopt to replace DaisyUI `progress` usage.                 |
| Radio Group   | [preview/src/components/radio_group](https://github.com/DioxusLabs/components/tree/main/preview/src/components/radio_group)     | —                                                      | Missing                                                          | Adopt to replace DaisyUI radio usage.                      |
| Scroll Area   | [preview/src/components/scroll_area](https://github.com/DioxusLabs/components/tree/main/preview/src/components/scroll_area)     | —                                                      | Missing                                                          | Adopt for scrollable panes with consistent styling.        |
| Select        | [preview/src/components/select](https://github.com/DioxusLabs/components/tree/main/preview/src/components/select)               | `crates/tasklens-ui/src/components/select/`            | Wrapper/Modified (DaisyUI classes)                               | Re-vendor and restore upstream styling.                    |
| Separator     | [preview/src/components/separator](https://github.com/DioxusLabs/components/tree/main/preview/src/components/separator)         | —                                                      | Missing                                                          | Adopt where separators are used in menus or cards.         |
| Sheet         | [preview/src/components/sheet](https://github.com/DioxusLabs/components/tree/main/preview/src/components/sheet)                 | —                                                      | Missing                                                          | Adopt for slide-over panels if needed.                     |
| Sidebar       | [preview/src/components/sidebar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/sidebar)             | —                                                      | Missing                                                          | Adopt if sidebar navigation is added.                      |
| Skeleton      | [preview/src/components/skeleton](https://github.com/DioxusLabs/components/tree/main/preview/src/components/skeleton)           | —                                                      | Missing                                                          | Adopt to replace custom loading skeletons.                 |
| Slider        | [preview/src/components/slider](https://github.com/DioxusLabs/components/tree/main/preview/src/components/slider)               | `crates/tasklens-ui/src/components/slider/`            | Adopted (minor formatting diffs)                                 | Keep and re-sync to upstream formatting.                   |
| Switch        | [preview/src/components/switch](https://github.com/DioxusLabs/components/tree/main/preview/src/components/switch)               | —                                                      | Missing                                                          | Adopt to replace DaisyUI `toggle` usage.                   |
| Tabs          | [preview/src/components/tabs](https://github.com/DioxusLabs/components/tree/main/preview/src/components/tabs)                   | —                                                      | Missing                                                          | Adopt if tabbed navigation is needed.                      |
| Textarea      | [preview/src/components/textarea](https://github.com/DioxusLabs/components/tree/main/preview/src/components/textarea)           | —                                                      | Missing                                                          | Adopt to replace DaisyUI `textarea` usage.                 |
| Toast         | [preview/src/components/toast](https://github.com/DioxusLabs/components/tree/main/preview/src/components/toast)                 | —                                                      | Missing                                                          | Adopt to replace custom notifications.                     |
| Toggle        | [preview/src/components/toggle](https://github.com/DioxusLabs/components/tree/main/preview/src/components/toggle)               | —                                                      | Missing                                                          | Adopt for on/off UI instead of DaisyUI `toggle`.           |
| Toggle Group  | [preview/src/components/toggle_group](https://github.com/DioxusLabs/components/tree/main/preview/src/components/toggle_group)   | —                                                      | Missing                                                          | Adopt for multi-toggle controls.                           |
| Toolbar       | [preview/src/components/toolbar](https://github.com/DioxusLabs/components/tree/main/preview/src/components/toolbar)             | —                                                      | Missing                                                          | Adopt if toolbar patterns are introduced.                  |
| Tooltip       | [preview/src/components/tooltip](https://github.com/DioxusLabs/components/tree/main/preview/src/components/tooltip)             | —                                                      | Missing                                                          | Adopt to replace custom hover hints.                       |

## Divergence Summary

- Total upstream components: 38
- Adopted: 3 (Navbar, Popover, Slider)
- Wrapper/Modified: 4 (Calendar, Collapsible, Dialog, Select)
- Diverged replacements: 4 (Button, Checkbox, Date Picker, Input)
- Missing: 27

## Tailwind And DaisyUI Usage Audit

The app uses Tailwind and DaisyUI across both components and views. This has to be replaced by upstream component CSS plus a small `app.css` for layout and app-specific UI.
This is a categorized summary intended to map usage into replacement workstreams.

| Category           | Current Usage Examples                                                                                                       | Dioxus Components Coverage                                                                              | Gap And Plan                                                                              |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Layout And Spacing | `container`, `max-w-*`, `px-*`, `gap-*`, `grid`, `flex`, `justify-*` in `views/*`                                            | None (layout is app responsibility)                                                                     | Move to `app.css` utility classes or app layout components.                               |
| Typography         | `text-*`, `font-*`, `uppercase`, `tracking-*` in views and app components                                                    | Theme sets base font only                                                                               | Move to `app.css` heading and label styles, or app components (PageHeader, SectionTitle). |
| Color And Theme    | `bg-base-*`, `text-base-*`, `text-primary`, `badge-*` across app                                                             | Theme variables exist upstream                                                                          | Replace DaisyUI tokens with upstream CSS variables in `app.css`.                          |
| Interactive States | `hover:*`, `focus:*`, `group-hover:*` sprinkled in views                                                                     | Many upstream components include hover/focus styles                                                     | Keep in component CSS where possible, add app CSS for app-specific interactions.          |
| Component Skins    | `btn`, `card`, `badge`, `input`, `select`, `textarea`, `toggle`, `range`, `progress`, `dropdown`, `modal`, `menu`, `loading` | Buttons, Inputs, Select, Textarea, Toggle, Progress, Dialog, Dropdown Menu, Toast, Tooltip, Card, Badge | Replace DaisyUI class usage with upstream components and styles.                          |
| App-Specific UI    | Task rows, task editor layout, navbar controls, score trace cards                                                            | Partial (Navbar, Card, Badge)                                                                           | Move app UI to `app_components` and style via `app.css` + upstream components.            |

## Gap Analysis Against Dioxus Components

- Buttons and toggles: upstream Button, Toggle, ToggleGroup can replace `btn`, `toggle`, and `join` patterns.
- Inputs: upstream Input, Checkbox, RadioGroup, Select, Textarea, Label should replace DaisyUI input classes.
- Overlays: upstream Dialog, AlertDialog, Popover, DropdownMenu, ContextMenu, Tooltip, Toast can replace `modal`, `dropdown`, and `menu` usage.
- Layout containers: upstream Card, Separator, ScrollArea, Accordion, Collapsible can replace card-like layouts and collapsible UI.
- Feedback: upstream Badge, Progress, Skeleton can replace DaisyUI badges and progress bars.
- Navigation: upstream Navbar, Tabs, Menubar, Sidebar can replace navigation styling patterns.
- Date and time: upstream Calendar and DatePicker should replace the HTML date input and custom calendar wrappers.
- Remaining gaps: layout primitives, typography variants, and app-specific flows must be handled in `app.css` or `app_components`.

## Module Boundary And Naming

- Require `crates/tasklens-ui/src/dioxus_components` as the vendored Dioxus Components module.
- Create `crates/tasklens-ui/src/app_components` for app-specific UI and layout components.
- App-specific components have been moved out of the legacy `components` module (examples: `task_row`, `task_editor`, `app_navbar`, `sync_indicator`, `empty_state`).
- Keep `crates/tasklens-ui/src/components` as a temporary compatibility shim until imports are fully migrated.
- Reserve `dioxus_components` for upstream-sourced components and minimal wrappers.
- Prefer `dioxus_components` over `dioxus-components`: Rust module names are idiomatically `snake_case`, while hyphenated paths require non-idiomatic `#[path = "..."]` indirection.

Required vendor config (`dx components add` defaults to `src/components`):

```toml
# crates/tasklens-ui/dioxus-vendor-components.toml
module_path = "src/dioxus_components"
components = ["button"]

[registry]
git = "https://github.com/DioxusLabs/components"
rev = "<pinned-upstream-rev>"
```

- `cargo xtask update-dioxus-components` reads `module_path` and `registry` from this file and passes them via `--module-path`, `--git`, and `--rev`.
- Because `dx components add` updates `<module_path>/mod.rs`, this repo must keep generated module registration in `src/dioxus_components/mod.rs`.

## Dependency Strategy For Dioxus Primitives

Options to evaluate:

- Git dependency pinned to a specific upstream commit.
  - Pros: Simple, no local vendoring needed.
  - Cons: Harder to patch primitives locally, can drift from vendored components.
- Path dependency to a vendored upstream snapshot.
  - Pros: Primitives and components stay in lockstep, easier to patch.
  - Cons: Requires a vendored copy of the upstream repo.

Recommendation: If we adopt a vendor workflow (subtree or vendor branch), use a path dependency to the vendored `primitives` crate to keep versions aligned. Otherwise, pin the git dependency to the same commit as the vendored components and update together.

Version compatibility note:

- The upstream repo currently targets Dioxus `0.7.0`, while this workspace uses `0.7.3`.
- We should verify compatibility when syncing upstream and prefer keeping versions aligned if issues arise.

## CSS Strategy for Removing Tailwind

Chosen approach: upstream per-component CSS + global upstream theme + minimal `app.css`.

Why this approach:

- It preserves upstream styling and minimizes diff churn.
- It avoids re-encoding upstream CSS into utility classes.
- It keeps updates and merge diffs small.

Alternatives considered:

- Bundled `components.css`: fewer link tags, but harder diffing and more merge noise.
- CSS Modules or scoped styles: reduces leakage, but diverges from upstream and increases churn.
- Inline styles: simple tooling, but farthest from upstream and hard to maintain.

Guidelines:

- Keep the upstream `dx-components-theme.css` pristine.
- Put app-specific layout and typography in `app.css`.
- Avoid adding Tailwind or DaisyUI classes in components or views.

## Build And Asset Pipeline Changes

- Remove the special `tailwind.css` build path and the `document::Stylesheet` reference to `assets/tailwind.css`.
- Add `assets/app.css` and link it in `main.rs` using `document::Stylesheet` or `document::Link`.
- Ensure upstream component `style.css` files are linked from each component (via `document::Link`).
- Keep `dx-components-theme.css` as a single global import.

## DX Installation Processing Implications

`dx components add` is a light processing pipeline, not a plain file copy.

- It resolves builtin and third-party component dependencies before install.
- It copies component files while applying manifest `exclude` rules.
- It copies declared global assets into the configured asset directory.
- It runs `cargo add` for declared Rust dependencies.
- It updates `<components_dir>/mod.rs` with missing `pub mod ...` lines (default: `src/components/mod.rs`).
- It can target configured paths from `Dioxus.toml` or `--module-path`, and can overwrite with `--force`.
- In this repo, xtask must pass `--module-path`, `--git`, and `--rev` from `dioxus-vendor-components.toml` so installs remain deterministic and generated module updates land in `src/dioxus_components/mod.rs`.

Impact on strategy:

- Any strategy that bypasses `dx components add` must reimplement these transforms to get equivalent installed output.
- The risk of drift is not only styling/API drift but also installer-behavior drift (dependencies, assets, module registration).

## Upstream Tracking Strategy

We need a workflow that supports intelligent merges, local patch maintenance, and high fidelity to `dx components add` output.

Options:

- Submodule plus direct file vendoring
  - Pros: Preserves upstream history cleanly.
  - Cons: Still needs a separate install step; local patch flow is awkward; high operational friction for daily component work.
- Subtree of upstream source with direct copy into `src/dioxus_components`
  - Pros: Best in-tree upstream history and straightforward upstream diffs.
  - Cons: Lowest installer fidelity unless we reimplement `dx` processing; easy to miss assets/deps/module updates.
- Pristine vendor branch driven by `dx components add`
  - Pros: Highest fidelity to real installed output; dependency and asset handling stay aligned with `dx`; local patches can be maintained on top of vendor merges.
  - Cons: Does not preserve line-level upstream history in the installed tree; requires disciplined vendor-branch updates.
- Hybrid mirror plus installer branch
  - Pros: Keeps upstream history in a mirror subtree while still producing installer-faithful output via `dx` on a vendor branch.
  - Cons: More moving parts and branch management overhead.

Pristine vendor branch approach (in prose):

- Create an orphan vendor branch that contains only installer-produced component state.
- Configure registry `git`/`rev` and `module_path` in `dioxus-vendor-components.toml` so xtask passes `--module-path`, `--git`, and `--rev` before any vendoring run.
- Run `dx components add` for the selected components into `crates/tasklens-ui/src/dioxus_components`.
- Commit that exact output as the pristine vendor snapshot.
- Merge vendor snapshots into the working branch; keep app patches as follow-on commits.
- Update by repeating on the vendor branch and merging forward.

Recommendation: Prefer the pristine vendor branch driven by `dx components add`.

- This best satisfies the new constraint that installer processing matters for correctness.
- It still supports intelligent merges and local patch maintenance through repeatable vendor snapshots.
- If preserving upstream source history is still required for audit, add a separate read-only upstream mirror branch and keep it out of the runtime install path.

## Component Acquisition Workflow

- `dx components add` should be the source of truth for what lands in `src/dioxus_components` because it applies installer processing (deps, assets, excludes, module updates).
- Direct copying from upstream source is useful for audit and review, but not sufficient as the install mechanism.

Recommendation:

- Use `dx components add` as the primary acquisition step on the pristine vendor branch.
- Pin registry `git` and `rev` so installs are deterministic.
- Set `module_path = "src/dioxus_components"` and registry `git`/`rev` in `dioxus-vendor-components.toml`; xtask will pass matching installer arguments consistently.
- Keep local patches on top of merged vendor snapshots.

## Representative Diffs

- Button: local uses DaisyUI classes; upstream uses CSS variables and `style.css` with `data-style` variants.
- Checkbox: local is a plain input with DaisyUI classes; upstream wraps `dioxus-primitives::checkbox` and renders an indicator SVG.
- Input: local API is trimmed and DaisyUI-based; upstream supports a broader attribute surface and uses `style.css`.
- Dialog: local is styled as a DaisyUI modal; upstream uses dedicated dialog CSS and attributes for accessibility.
- Date Picker: local exports an HTML `<input type="date">`; upstream provides a full picker with calendar and range support.

## Migration Plan And Priorities

- Implement the recommended upstream tracking strategy: pristine vendor branch driven by `dx components add` and pinned registry revision.
- Split modules: `dioxus_components` for vendored Dioxus Components, `app_components` for app UI.
- De-tailwind: remove Tailwind and DaisyUI classes, add `app.css` for layout and typography.
- Re-vendor and replace diverged components first: Button, Input, Checkbox, Date Picker.
- Align existing wrappers: Calendar, Collapsible, Select, Dialog.
- Adopt missing upstream components as needed (Badge, Card, Progress, Textarea, Toggle, Tooltip, Toast, etc).
- Migrate app-specific UI to `app_components` and style via `app.css` + upstream components.
- Update the inventory and divergence summary as components are aligned.

## Checklist

See [dioxus-components-migration.todo.md](dioxus-components-migration.todo.md) for the actionable checklist.
