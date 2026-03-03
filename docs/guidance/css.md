# CSS Style Guide

This document outlines the conventions, requirements, and best practices for
authoring primary stylesheets and establishing app-local design constraints
within `tasklens-ui`.

> [!NOTE]
> This guide focuses on authoring **app-level CSS**. If you are modifying, overriding, or customizing the behavior of **vendored Dioxus components**, you must also adhere to our component-specific guidelines:
>
> - **[Dioxus Vendored Styling Guidance](./dioxus-vendored-styling.md)**: Rules for writing CSS that overrides or extends vendored components.
> - **[Dioxus Vendored Customization Guidance](./dioxus-vendored-customization.md)**: Rules for wrapping, extending, or altering the behavior of vendored components.

All new app CSS and UI components must adhere to the following guidelines.

## Two Kinds of Rules

This guide contains two distinct categories of rules. Understanding the
difference matters when deciding what applies to vendored code.

**Authoring rules** govern how we write and organize CSS source code: token
usage, naming conventions, CSS modules, layout patterns, and inline-style
restrictions (Sections 1, 2, 4, 5, 6, 7). These apply to **app-owned source
files** — they constrain _implementation_, not rendered output. Vendored source
files are exempt from authoring rules because we treat them as upstream code
with its own conventions.

**Runtime rules** define requirements that must hold when the application runs
in the browser: minimum touch-target sizes, accessible interactive areas, and
visual-vs-interactive sizing (Section 3). These are evaluated against the
**rendered output**, regardless of which source file produced it.

Do not infer runtime non-compliance solely from source-level dimensions on a
vendored primitive (for example, a 1rem checkbox). If a vendored primitive is
visually small, satisfy runtime requirements with app-owned wrappers/scoped CSS
that provide the effective hit region.

Use this decision tree:

1. For app-owned CSS, apply both authoring and runtime rules from this guide.
2. For styling changes that touch vendored paths
   (`src/dioxus_components/**`, `assets/dx-components-theme.css`), use
   **[Dioxus Vendored Styling Guidance](./dioxus-vendored-styling.md)**.
3. If the rendered output does not meet a runtime rule, prefer a layered
   architecture first (adapter + wrapper markup + scoped CSS), but use
   judgment — see the
   **[Vendored Styling Guidance](./dioxus-vendored-styling.md#deliberate-vs-gratuitous)**
   for when a direct vendored edit is the simpler choice.

## 1. CSS Resets

A consistent baseline across all browsers is required.

- **Global Reset**: We use an app-level default normalization (found in `assets/app.css`), which handles resetting typography, lists, and form controls.
- **Box Sizing**: Elements should use `box-sizing: border-box`.
- **Avoid Ad-Hoc Resets**: Do not attempt to reset margins, padding, or typography arbitrarily in component CSS files. Trust the global reset and explicitly style the element instead.

## 2. Design Tokens and Theming

This project utilizes a token-driven design system powered by CSS Custom
Properties (variables).

- **No New Hardcoded Colors (Default)**: In app-owned CSS, use semantic tokens
  instead of raw color literals.
- **Temporary Exception (Time-Boxed)**: A hardcoded color is acceptable only as
  a short-lived bridge when no suitable token exists yet. It must include:
  1. A nearby `TODO` comment with an issue/task id.
  2. The intended replacement token in the comment.
  3. Removal in the next change that introduces the token.
- **Theme Tokens** (`assets/dx-components-theme.css`): Contains core palette values (e.g., `--primary-color-2`, `--secondary-color-4`) and handles dark/light mode mapping via `data-theme` or `prefers-color-scheme`.
- **App Semantic Tokens** (`assets/app.css`): Whenever possible, use semantic tokens mapped to specific interface roles rather than raw colors:
  - **Surfaces**: `--app_surface`, `--app_surface-muted`, `--app_surface-subtle`
  - **Text**: `--app_text`, `--app_text-muted`, `--app_text-subtle`, `--app_accent`
  - **Borders**: `--app_border`, `--app_border-strong`

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
  acceptable if surrounding app-owned composition provides a compliant rendered
  hitbox and activation behavior.

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

_Allowed pattern:_ a vendored checkbox remains `1rem` visually, while an
app-owned wrapper element provides a 44x44 effective interactive area.

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

- **Global Utilities**: Any class inside `app.css` intended for general use across multiple component trees **must** be prefixed with `app_` (e.g., `.app_rounded`, `.app_stack_4`, `.app_text_sm`).
- **Legacy Utility Migration**: Existing non-`app_` utility classes in
  `app.css` are legacy. Do not introduce new non-`app_` utilities; migrate
  legacy names opportunistically when touching the same area, and track larger
  renames in a follow-up task.
- **Component Classes**: Components should use descriptive class names relevant to their local domain (e.g., `.row_root`, `.urgency_badge`). Do not use raw tag selectors outside of global resets.
- **Transitions**: To maintain a cohesive feel, reuse the standard `.app_transition` class for hover or focus state animations rather than defining custom transition rates and easing everywhere.

## 5. CSS Modules for App-Specific Components

When creating app-specific Dioxus components (`crates/tasklens-ui/src/app_components`), encapsulate your styles using the Dioxus `css_module` macro. This prevents selector collisions and provides compile-time checking for CSS classes.

**Best Practices:**

1. **Colocation**: Place a `.css` file next to your component's `.rs` file (e.g., `task_row.css` and `task_row.rs`).
2. **Setup the Macro**: Define a `Styles` struct using the `#[css_module]` macro.
3. **Class Extraction**: Use `class: Styles::your_class_name` instead of raw strings.
4. **CSS Naming Scheme**: Prefer `snake_case` for your CSS class names (e.g., `.task_title`). Since the Dioxus macro exposes these as Rust fields/constants, writing CSS in `snake_case` results in a perfect 1:1 name mapping without relying on the macro's hidden case-conversion magic.

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
- Avoid absolute positioning unless building overlays, or floating menus. Use Flexbox's `gap` property over arbitrary margin assignments where possible.

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
