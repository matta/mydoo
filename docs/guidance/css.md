# CSS Style Guide

This document outlines the conventions, requirements, and best practices for
authoring primary stylesheets and establishing project-local design constraints
within `tasklens-ui`.

> [!NOTE]
> This guide focuses on authoring **Project-authored CSS**. If you are
> modifying, overriding, or customizing the behavior of **Vendored Dioxus
> CSS**, you must also adhere to our component-specific guidelines:
>
> - **[Dioxus Vendored Styling Guidance](./dioxus-vendored-styling.md)**: Rules for writing CSS that overrides or extends vendored components.
> - **[Dioxus Vendored Customization Guidance](./dioxus-vendored-customization.md)**: Rules for wrapping, extending, or altering the behavior of vendored components.

All new Project-authored CSS and UI components must adhere to the following
guidelines.

## Scope Terminology

- **Vendored Dioxus CSS**: Upstream/vendor-controlled styles in
  `src/dioxus_components/**` and `assets/dx-components-theme.css`.
- **Component Module CSS**: Project-authored CSS Modules, typically in
  `src/app_components/**/*.css` (or other paths referenced by `#[css_module]`).
- **Global Project CSS**: Project-authored, non-module global stylesheets
  (currently `assets/app.css`).
- **Project-authored CSS**: Umbrella term for Component Module CSS + Global
  Project CSS.

## Two Kinds of Rules

This guide contains two distinct categories of rules. Understanding the
difference matters when deciding what applies to vendored code.

**Authoring rules** govern how we write and organize CSS source code: token
usage, naming conventions, CSS modules, layout patterns, and inline-style
restrictions (Sections 1, 2, 4, 5, 6, 7, 8). These apply to
**Project-authored CSS** — they constrain _implementation_, not rendered
output. Vendored source files are exempt from authoring rules because we treat
them as upstream code with its own conventions.

**Runtime rules** define requirements that must hold when the application runs
in the browser: minimum touch-target sizes, accessible interactive areas, and
visual-vs-interactive sizing (Section 3). These are evaluated against the
**rendered output**, regardless of which source file produced it.

Do not infer runtime non-compliance solely from source-level dimensions on a
vendored primitive (for example, a 1rem checkbox). If a vendored primitive is
visually small, satisfy runtime requirements with Project-authored
wrappers/scoped CSS that provide the effective hit region.

Use this decision tree:

1. For Project-authored CSS, apply both authoring and runtime rules from this
   guide.
2. For styling changes that touch Vendored Dioxus CSS, use
   **[Dioxus Vendored Styling Guidance](./dioxus-vendored-styling.md)**.
3. If the rendered output does not meet a runtime rule, prefer a layered
   architecture first (adapter + wrapper markup + scoped CSS), but use
   judgment — see the
   **[Vendored Styling Guidance](./dioxus-vendored-styling.md#deliberate-vs-gratuitous)**
   for when a direct vendored edit is the simpler choice.

## 1. CSS Resets

A consistent baseline across all browsers is required.

- **Global Reset**: We use a Global Project CSS default normalization (found in
  `assets/app.css`), which handles resetting typography, lists, and form
  controls.
- **Box Sizing**: Elements should use `box-sizing: border-box`.
- **Avoid Ad-Hoc Resets**: Do not attempt to reset margins, padding, or typography arbitrarily in component CSS files. Trust the global reset and explicitly style the element instead.

## 2. Design Tokens and Theming

This project utilizes a token-driven design system powered by CSS Custom
Properties (variables).

- **No New Hardcoded Colors (Default)**: In Project-authored CSS, use semantic
  tokens instead of raw color literals.
- **Temporary Exception (Time-Boxed)**: A hardcoded color is acceptable only as
  a short-lived bridge when no suitable token exists yet. It must include:
  1. A nearby `TODO` comment with an issue/task id.
  2. The intended replacement token in the comment.
  3. Removal in the next change that introduces the token.
- **Theme Tokens** (`assets/dx-components-theme.css`): Contains core palette values (e.g., `--primary-color-2`, `--secondary-color-4`) and handles dark/light mode mapping via `data-theme` or `prefers-color-scheme`.
- **Global Semantic Tokens** (`assets/app.css`): Whenever possible, use semantic
  tokens mapped to specific interface roles rather than raw colors:
  - **Surfaces**: `--app_surface`, `--app_surface_muted`,
    `--app_surface_subtle`
  - **Text**: `--app_text`, `--app_text_muted`, `--app_text_subtle`,
    `--app_accent`
  - **Borders**: `--app_border`, `--app_border_strong`
  - Existing hyphenated semantic token names are legacy; migrate them
    opportunistically during related work. Do not introduce new hyphenated
    names.

## 3. Sizing and Touch Areas (Accessibility)

Building inclusive, usable interfaces requires explicit attention to touch target sizing. Mobile platforms dictate strict requirements for tap areas.

- **Minimum Touch Targets**: Any interactive action in the rendered UI **MUST**
  expose an actionable region of at least **44x44 pixels** (`2.75rem` square).
- **Preferred Touch Targets**: For primary mobile actions, strive for **48x48 pixels** (`3rem x 3rem`).
- **Visual vs. Interactive Sizing**: If the visual footprint of a control (like a small checkmark or chevron) must remain small, you **must expand the hit area**. Achieve this using:
  - Generous `padding` with `background-color: transparent`.
  - A flex container wrapper with explicit `width` and `height` properties.
  - Sizing techniques like `min-height: 3rem;` on interactive rows.
- **Vendored Primitive Nuance**: A visually small vendored primitive is
  acceptable if surrounding Project-authored composition provides a compliant
  rendered hitbox and activation behavior.

_Example pattern:_

```css
.chevron_container {
  width: 2.75rem; /* 44px - standard touch target width */
  display: flex;
  justify-content: center;
  align-items: center;
}

.checkbox_hit_target {
  width: 2.75rem; /* 44px minimum touch target */
  height: 2.75rem;
  display: flex;
  justify-content: center;
  align-items: center;
}
```

_Allowed pattern:_ a vendored checkbox remains `1rem` visually, while a
Project-authored wrapper element provides a 44x44 effective interactive area.

### 3.1 Runtime Verification and Review Evidence

Runtime rules are verified in the browser, not by CSS source inspection alone.

- **Primary check**: Use E2E/manual runtime validation to confirm effective
  actionable region size and behavior.
- **Static review constraint**: If runtime evidence is absent, reviewers should
  label the item **Needs runtime verification** instead of asserting definitive
  non-compliance from source-only inspection.
- **PR evidence expectation**: For accessibility-affecting UI changes, include
  either an E2E assertion or concise manual evidence that the effective hitbox
  meets the minimum size requirement.

## 4. Utilities and Naming Conventions

- **Naming scope**: Naming rules in this section apply to
  Project-authored CSS (Component Module CSS and Global Project CSS). Vendored
  Dioxus CSS is exempt.
- **Underscore naming style**: In Project-authored CSS, use underscore-style
  names for selectors and custom properties (for example, `.task_row`,
  `--task_row_indent`, `--app_spacing_sm`). Do not introduce new kebab-case
  names. This keeps grep alignment with Rust/CSS-module identifiers.
- **`app_` prefix scope**: The `app_` prefix is required for shared global names
  declared in Global Project CSS (`assets/app.css`):
  1. Global utility classes (for example, `.app_rounded`, `.app_stack_md`).
  2. Global app custom properties (for example, `--app_spacing_sm`).
- **Component Module CSS local names**: In Component Module CSS,
  `app_` is not required. Prefer descriptive domain names
  (for example, `.row_root`, `.urgency_badge`).
- **Legacy naming migration**: Existing non-conforming names (including
  non-`app_` global utilities, numeric spacing utility suffixes, and hyphenated
  project-authored names) are legacy. Do not add new names in those styles.
  Migrate opportunistically during related work, and track larger renames in
  follow-up tasks.
- **Transitions**: To maintain a cohesive feel, reuse the standard `.app_transition` class for hover or focus state animations rather than defining custom transition rates and easing everywhere.

## 5. Component Module CSS for Project Components

When creating project-specific Dioxus components
(`crates/tasklens-ui/src/app_components`), encapsulate your styles using the
Dioxus `css_module` macro. This prevents selector collisions and provides
compile-time checking for CSS classes.

**Best Practices:**

1. **Colocation**: Place a `.css` file next to your component's `.rs` file (e.g., `task_row.css` and `task_row.rs`).
2. **Setup the Macro**: Define a `Styles` struct using the `#[css_module]` macro.
3. **Class Extraction**: Use `class: Styles::your_class_name` instead of raw strings.
4. **Naming Policy**: Follow Section 4 naming rules in module CSS
   (underscore-style names; no required `app_` prefix for local classes).

**Refinement to Current Practice:**
Currently, `#[css_module(...)] struct Styles;` is often placed _inside_ the component function body.
While this is acceptable for single-component files, **we recommend defining the `Styles` struct at the module scope (outside the function)**.

_Why?_

- It allows helper UI functions and sub-components in the same file to share the exact same style definitions without needing to pass class strings manually or re-declare the macro.
- It clarifies that style definitions are static and compile-time rather than re-evaluated on every component render.

_Example Pattern:_

```rust
use dioxus::prelude::*;

// 1. Module-level definition ensures availability to sub-components
#[css_module("/src/app_components/my_component.css")]
struct Styles;

#[component]
pub fn MyComponent() -> Element {
    let is_active = true;

    // 2. Combine classes cleanly if needed using format! or simple conditionals
    let container_class = if is_active {
        format!("{} {}", Styles::container, Styles::active)
    } else {
        Styles::container.to_string()
    };

    rsx! {
        div {
            class: "{container_class}",
            // 3. Direct access via Styles
            span {
                class: Styles::text_label,
                "Hello World"
            }
        }
    }
}
```

## 6. Layout Engine

- Prefer **Flexbox** for 1D layouts and **CSS Grid** for 2D layouts.
- Avoid absolute positioning unless building overlays, or floating menus.
- For flow spacing policy (`gap` over sibling margins for stack/row/grid
  layouts), follow Section 8.2.

## 7. Avoid Inline Styles For Concrete Properties

Do not set concrete visual properties inline (for example, `padding-left`,
`color`, `width`, `background-color`) within Dioxus components.

Inline styles are allowed only for passing dynamic CSS custom properties
(`--*`) that are then consumed by classes in stylesheet files.

**❌ Bad** — setting concrete properties inline:

```rust
div {
    style: "padding-left: {indentation}px;",
    "Content"
}
```

**✅ Good** — passing the dynamic value as a CSS variable, with the property
rule in the stylesheet:

```rust
// my_component.rs
div {
    class: Styles::indented_row,
    style: "--indent: {indentation}px;",
    "Content"
}
```

```css
/* my_component.css */
.indented_row {
  padding-left: var(--indent);
}
```

## 8. Layout Expression and Spacing Consistency

This section defines how we express layout in Project-authored UI so that spacing and
structure stay consistent as the component library grows.
It is policy for all new Project-authored UI work. Legacy layout patterns
should be migrated opportunistically during related work.

### 8.1 Design Language: Spacing Scale First

Consistency comes primarily from a **single spacing scale** (tokens) and a
policy that components follow it.

- **Prefer tokenized spacing**: Use the app spacing token set in
  `assets/app.css` and keep new spacing values on a semantic t-shirt scale with
  the `--app_spacing_` prefix (`--app_spacing_xs`, `--app_spacing_sm`,
  `--app_spacing_md`, `--app_spacing_lg`, `--app_spacing_xl`). Do not introduce
  numeric spacing token suffixes for new Project-authored tokens.
- **Avoid one-off spacing**: Do not introduce arbitrary values like `13px`,
  `18px`, `22px` for margins/padding/gaps unless you are explicitly matching a
  runtime requirement (e.g., the 44px touch target) or bridging to a future
  token with a time-boxed exception and a TODO.

### 8.2 Prefer `gap` Over Margins for Flow Layout

Where possible, express spacing between siblings using `gap`:

- **Stacks (vertical flow)**: `display: flex; flex-direction: column; gap: …`
- **Clusters/Rows (horizontal flow)**: `display: flex; flex-direction: row; gap: …`
- **Grids**: `display: grid; gap: …`

Using `gap` avoids margin-collapsing surprises and reduces the need for
selector-specific spacing rules.

### 8.3 Layout Primitives (Recommended Set)

Most app screens can be expressed using a small set of primitives. Prefer these
before inventing bespoke page-specific layout classes:

- **Container**: max width + horizontal padding
- **Stack (Done)**: vertical layout with `gap`
- **Cluster/Row**: horizontal layout with `gap`, optional wrap and alignment
- **Grid**: 2D layout with responsive columns
- **Spacer/Divider (optional)**: explicit separation when flow layout is not enough

### 8.4 Where Spacing Lives (Ownership Policy)

To keep layouts predictable:

- **Parents own outer spacing**: placement, page/grid/stack spacing, section
  padding, and distance between components.
- **Components own inner spacing**: padding and internal arrangement of the
  component’s own sub-elements.
- **Avoid “mystery margins” on leaf components**: Leaf UI components should not
  generally add external margins that surprise their parent container.

This policy makes it easier to rearrange layouts without hunting for stray
margins across unrelated modules.

### 8.5 Dioxus Composition: Primitives as Components vs Global Utilities

Because this codebase uses Component Module CSS for project components, we have two valid
ways to express layout consistently without falling back into “utility class
clutter”.

**Option A (Preferred): Layout primitives as small Dioxus components**

Create a tiny set of Project-authored primitives in `src/app_components/layout/**`
(e.g., `Stack`, `Row`, `Grid`, `Container`). These components should:

- Render minimal markup (`div` / `section`) with predictable classes.
- Accept a _small_ set of parameters (e.g., `gap`, `align`, `justify`, `wrap`,
  `pad`, `max_width`) that map to tokens.
- Avoid becoming a mini-framework. Keep the API intentionally small and aligned
  with our token scale.

For example, your Dioxus code would remain declarative and clean:

```rust
rsx! {
    Stack { gap: "md",
        h1 { "Title" }
        Row { gap: "sm", justify: "end",
            Button { "Cancel" }
            Button { "Save" }
        }
    }
}
```

This approach keeps RSX trees readable and enforces the spacing scale by
construction.

**Option B: A small set of global layout utilities**

If primitives-as-components feels too heavy for a specific area, define a small
set of global utilities in `assets/app.css`, **prefixed with `app_`** (per the
utilities rules). Keep them few and stable, e.g.:

- `.app_container`
- `.app_stack_{size}` (e.g., `.app_stack_md`) or `.app_stack` + a CSS variable
  `--app_stack_gap`
- `.app_row_{size}` (e.g., `.app_row_md`)
- `.app_grid_{variant}`

Do **not** reintroduce a large “utility matrix”. If you find yourself needing
dozens of near-duplicates, prefer Option A or a local CSS Module.

**Default choice**: Use **Option A** for common structural layout. Use Option B
sparingly for simple screen-level composition or when prototyping.

### 8.6 Responsive Layout Guidance (Minimal and Predictable)

- Prefer **Grid** for responsive “card” layouts and **flex wrap** for small
  horizontal clusters.
- Prefer **a few named breakpoints** (if present) rather than continuous
  one-off media queries per component.
- Avoid embedding responsive behavior deep inside leaf components unless the
  component’s responsibility is inherently responsive (e.g., a navigation bar).

### 8.7 No Fallback Defaults for Guaranteed Variables

Do not provide fallback default values for CSS variables that should always
resolve (for example, `padding-top: max(0.5rem, var(--app_safe_top))` or
`var(--app_spacing_sm, 0.5rem)`).

When a variable is globally defined (like safe-area insets or app tokens),
local fallbacks merely serve to hide bugs. Trust the variable to resolve correctly.
If an element lacks appropriate spacing when the variable resolves to `0px`,
fix the parent layout container rather than masking it with a default.

## 9. Working With Vendored Components

When composing with vendored Dioxus components:

- Prefer theme/token alignment (CSS variables) over selector overrides.
- If a vendored primitive is visually small, satisfy runtime touch targets with
  **Project-authored wrappers** that provide the effective hit region (see Section 3).
