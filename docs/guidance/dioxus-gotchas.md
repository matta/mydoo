# Dioxus Gotchas

This document records important framework quirks, bugs, and non-obvious behaviors in Dioxus. It serves as a living anti-pattern guide to save developers from lengthy debugging sessions.

The agent-facing summary for this repository lives in
`docs/guidance/agents-dioxus.md`. Keep detailed framework caveats and examples
here; keep agent instructions concise in that file.

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
// ✅ CORRECT: Return only dynamic classes from helper logic.
fn dynamic_class() -> String {
    format!("{}", Styles::gap_md.inner)
}

#[component]
pub fn Stack(attributes: Vec<Attribute>, children: Element) -> Element {
    let class_name = dynamic_class();
    let base = attributes!(div {
        class: Styles::stack,     // 1. Explicitly notify Dioxus that the CSS module is used.
        class: "{class_name}",    // 2. Add derived dynamic classes without duplication.
    });
    // ...
}
```

> **Alternative Note:** If you cannot modify the macro logic for some reason, you can manually force the stylesheet to load using an explicit `document::Link { href: asset!("/path/to/file.css") }`. However, using the native `class: Styles::property` approach is officially idiomatic and avoids manually managing paths.

## 2. Spread Attributes and Class Merging in `rsx!`

When a component accepts passthrough attributes (`#[props(extends = ...)]`), direct `..attributes` spread in `rsx!` is not always equivalent to explicitly merging attributes first.

In this document, **explicit merge** means:

1. Build a base attribute set for the component-owned defaults (`class`, `role`, etc.) with `attributes!`.
2. Merge base attributes with caller-provided attributes using `merge_attributes`.
3. Spread the merged result into the final element.

### The Pitfall: Assuming Spread Fully Merges Classes

This pattern looks concise, but has two common failure modes in Dioxus 0.7.x:

```rust
// ❌ Fragile: may fail to compile or merge classes as expected
rsx! {
    div {
        class: Styles::stack,
        class: gap.class_name(),
        class: align.class_name(),
        ..attributes,
        {children}
    }
}
```

1. Dioxus macro limitation:
   builds can fail with `Cannot merge non-fmt literals` when multiple `class:` entries are non-format values in this shape.
2. Class semantics mismatch:
   even when rewritten to compile, caller `class` passed via spread may render as a separate `class="..."` attribute in SSR output instead of a single merged class list.

### The Safe Pattern: Build + Merge Explicitly

Use `attributes!` + `merge_attributes` to guarantee one normalized class list and stable passthrough behavior.

```rust
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
```

```rust
let base = attributes!(div {
    class: Styles::stack,
    class: "{gap.class_name()} {align.class_name()}",
});
let merged = merge_attributes(vec![base, attributes]);

rsx! {
    div {
        ..merged,
        {children}
    }
}
```

### What "non-fmt literals" Means

The Dioxus RSX macro has a merge path for repeated attributes (like multiple `class:`). In this path, values often need to be format-string compatible. These can fail:

```rust
// Can trigger: "Cannot merge non-fmt literals"
class: Styles::stack,
class: gap.class_name(),
class: align.class_name(),
```

A format-string expression is safer for repeated class composition:

```rust
class: "{Styles::stack} {gap.class_name()} {align.class_name()}",
```

Even with the format-string workaround, `..attributes` spread still may not match `merge_attributes` semantics for caller-provided `class` values. For reusable primitives, prefer explicit merge.

### Testing Gotcha: `render_element` vs Full Component Rendering

`dioxus_ssr::render_element` is useful for plain element snippets that do not rely on component runtime behavior. For component-level equality checks (especially with props/spread), render a `VirtualDom` with `dioxus_ssr::render(&dom)` so the component runs inside a Dioxus runtime and emitted markup matches real behavior.
