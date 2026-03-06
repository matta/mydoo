# Dioxus Gotchas

This document records important framework quirks, bugs, and non-obvious behaviors in Dioxus. It serves as a living anti-pattern guide to save developers from lengthy debugging sessions.

## 1. CSS Module Tree-Shaking (`#[css_module]`)

When using the `#[css_module]` macro, Dioxus pairs your Rust component with a specific `.css` file. It reads the CSS, hashes the class names to prevent global collisions, and generates a wrapper `struct Styles;` that provides properties for each class.

### The Pitfall: Hidden Dependencies

Dioxus relies on **static AST parsing** to determine if a CSS module is actually used. If it does not statically detect the module being used, the Dioxus asset bundler (Manganis) will quietly "tree-shake" (delete) the CSS file from your final build. Your HTML elements will still get the hashed class names, but the browser will receive no CSS!

This most commonly happens when you dynamically compose classes using the `.inner` strings and never provide the wrapper type directly to a `class:` attribute:

```rust
// ❌ WRONG: Static analysis fails to see that `Styles` is used. The CSS is bundled out!
fn stack_class() -> String {
    format!("{} {}", Styles::stack.inner, Styles::gap_md.inner)
}

#[component]
pub fn Stack(attributes: Vec<Attribute>, children: Element) -> Element {
    let class_name = stack_class();
    let base = attributes!(div {
        class: "{class_name}", // Dioxus only sees a runtime string, not a module dependency!
    });
    // ...
}
```

### The Solution: Direct Wrapper Type Passing

To ensure Dioxus automatically injects your CSS module `<link>` into the HTML `<head>`, you **must** pass at least one property of the generated `Styles` wrapper struct natively into an `rsx!` or `attributes!` `class:` attribute.

A pattern that satisfies both dynamic composition and static analysis is to specify `class` multiple times. Dioxus will correctly merge them:

```rust
// ✅ CORRECT: The wrapper type is explicitly used, triggering CSS asset injection.
fn stack_class() -> String {
    format!("{} {}", Styles::stack.inner, Styles::gap_md.inner)
}

#[component]
pub fn Stack(attributes: Vec<Attribute>, children: Element) -> Element {
    let class_name = stack_class();
    let base = attributes!(div {
        class: Styles::stack,     // 1. Explicitly notify Dioxus that the CSS module is used!
        class: "{class_name}",    // 2. Safely apply your derived dynamic string!
    });
    // ...
}
```

> **Alternative Note:** If you cannot modify the macro logic for some reason, you can manually force the stylesheet to load using an explicit `document::Link { href: asset!("/path/to/file.css") }`. However, using the native `class: Styles::property` approach is officially idiomatic and avoids manually managing paths.
