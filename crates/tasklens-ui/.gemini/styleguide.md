# Tasklens UI Style Guide

This project follows a deliberate, product-driven approach to styling and component
customization. We prioritize maintainability and accessibility over rigid
"pristine" source requirements.

## 🚨 Critical Guidance Documents

When working on any UI or CSS changes, you **must** reference and follow these
documents:

1.  **[CSS Style Guide](../../../docs/guidance/css.md)**: Rules for authoring
    app-level CSS, tokens, and runtime accessibility (touch targets).
2.  **[Dioxus Vendored Styling Guidance](../../../docs/guidance/dioxus-vendored-styling.md)**:
    Rules for styling and overriding vendored components.
3.  **[Dioxus Vendored Customization Guidance](../../../docs/guidance/dioxus-vendored-customization.md)**:
    Rules for altering component behavior, APIs, or state.
4.  **[Dioxus Gotchas](../../../docs/guidance/dioxus-gotchas.md)**:
    Known framework pitfalls around CSS module injection, `rsx!` attribute/class
    merging, and SSR/component test behavior.

### Reader Instruction: Transinclusion

> [!IMPORTANT]
> **Instructions for AI Agents (including Gemini):**
> When you are tasked with making UI or CSS changes, you MUST transinclude (read
> or @handle) the relevant documents above to ensure your proposed changes are
> compliant with the project's current standards.
>
> - If you are writing new CSS or components, start with the **CSS Style Guide**.
> - If you are modifying anything in `src/dioxus_components/`, start with the
>   **Vendored Styling** and **Vendored Customization** guides.
> - If you are:
>   - touching `#[css_module]` components or class composition logic,
>   - using `..attributes` spread or `merge_attributes`,
>   - adding/changing SSR tests (`dioxus_ssr`) for component markup equality,
>     read **Dioxus Gotchas** before implementing changes.

## Core Philosophy

- **Deliberate vs Gratuitous**: Edit vendored code when it is the simplest, most
  maintainable solution for the product. Avoid gratuitous changes (renaming,
  reformatting) that create merge noise without clear value.
- **Runtime vs Authoring Rules**: Most CSS rules are authoring constraints on
  our source code. Sizing and touch targets are **runtime requirements**
  evaluated in the browser.
- **Vendored Primitives**: Surrounding app-owned composition (wrappers/padding) is the
  preferred way to satisfy runtime accessibility rules for small vendored
  primitives.
