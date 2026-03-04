# Dioxus Vendored Styling Guidance

This guidance defines styling policy for **all vendored Dioxus components** in
`tasklens-ui`.

Applies to:

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`
- app code that consumes vendored components

> [!NOTE]
> This guide is specifically for **styling** vendored components.
>
> - For general CSS authoring rules (tokens, touch targets, resets), see the **[CSS Style Guide](./css.md)**.
> - For the distinction between authoring rules and runtime rules (and which apply to vendored code), see **[Two Kinds of Rules](./css.md#two-kinds-of-rules)**.
> - For altering component behavior, APIs, or state, see the **[Dioxus Vendored Customization Guidance](./dioxus-vendored-customization.md)**.

## Philosophy

The upstream dioxus-components project is designed to be forked and edited. It
does not provide exhaustive customization hooks for every use case — editing
the source is the expected extension mechanism. We should do so as well, when
prudent.

Edit vendored code when it serves the product. Leave it alone when it doesn't.

The goal is not to keep vendored source pristine at all costs. The goal is to
**avoid gratuitous changes** — edits that create merge noise without clear
product value.

## Deliberate vs Gratuitous

This is the core distinction. Not "app-owned vs vendored", but "deliberate vs
gratuitous."

**Deliberate edits** are product-driven changes where modifying vendored source
is the simplest, most maintainable solution. When weighing whether a vendored
edit is the right approach, consider:

- Does the edit meaningfully simplify application code?
- Is the change surgical (additive or narrowly scoped, not restructuring)?
- Is merge-conflict risk low (stable area of the upstream codebase)?
- Would the alternative (wrapper, CSS override, etc.) create brittleness or
  disproportionate complexity?

If most answers are "yes", the vendored edit is likely the right call.

**Gratuitous edits** are changes that touch vendored files without clear product
justification:

> [!CAUTION]
> **Do not:**
>
> - Reformat vendored code to match app conventions (naming, spacing, etc.).
> - Rename vendored variables, classes, or functions for consistency.
> - Make drive-by style improvements to vendored files while working on
>   unrelated tasks.
> - Restructure vendored component internals unless the restructure _is_ the
>   task.

## Ownership Boundaries

### Vendored-owned

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`

### App-owned

- `crates/tasklens-ui/assets/app.css`
- `crates/tasklens-ui/src/app_components/**/*.css`
- `crates/tasklens-ui/src/views/**/*.css`
- app-level wrapper/adapter components around vendored primitives

## Approaches (Prefer Simplicity)

When you need to change how a vendored component looks, consider these
approaches and choose the one that produces the simplest, most maintainable
result:

1. **App tokens and semantic CSS**
   - Use app semantic tokens and app-owned styles, following the
     **[CSS Style Guide](./css.md)**.
   - Keep global overrides small and intentional.
2. **App wrapper/adapter component**
   - Encapsulate layout, spacing, variants, and repeated policy in app-owned
     wrappers.
   - Keep callsites free of one-off styling hacks.
3. **Scoped override from app code**
   - Use local wrapper classes and CSS modules.
   - Target vendored global classes with `:global(...)` when needed.
4. **Direct vendored edit**
   - When approaches 1–3 would create brittleness or disproportionate
     complexity, edit the vendored source directly.
   - Keep the edit surgical. Document rationale in the commit message.

These are not a strict escalation ladder — sometimes approach 4 is simpler and
more correct than approach 2. Use judgment.

## Styling Contract For Vendored Components

The following rules apply across the vendor tree:

1. Vendored class names in vendored components are part of the base style
   contract.
2. Many components forward `attributes`; passing `class` from callsites can
   replace or conflict with vendored base classes unless attributes are
   explicitly merged.
3. Therefore, do **not** rely on raw `class:` overrides on vendored components
   as a default customization method.
4. If a component explicitly merges base and caller attributes, treat that as an
   implementation detail, not a blanket policy for all components.

### Preferred extension patterns

1. Wrap the vendored component and expose a small app-level API.
2. Apply contextual layout via wrapper + scoped CSS module.
3. Use one-off inline style only as a temporary bridge.

**Anti-pattern (generic):**

```rust
VendorComponent { class: "w-full" }
```

**Preferred (wrapper + scoped override):**

```rust
div {
    class: Styles::container,
    VendorComponent {}
}
```

```css
.container :global(.vendor-base-class) {
  width: 100%;
}
```

## Theme File Policy

Goal state: keep `crates/tasklens-ui/assets/dx-components-theme.css` pristine
unless there is a clear product reason to change it.

A controlled edit is acceptable only if:

1. The need is truly global and cannot be solved cleanly in app-owned CSS.
2. The change aligns with vendored semantics rather than app-only divergence.
3. The rationale is documented in the commit message.

## Review Checklist

1. Is the vendored edit deliberate and product-driven, not gratuitous?
2. Is the change surgical and narrowly scoped?
3. Are overrides reversible and documented?
4. Did we avoid broad global selectors and repeated per-callsite hacks?
5. Does the change remain robust after a future `cargo xtask dx-components vendor` run?

## Optional Guardrails

Use these checks to spot risky styling changes:

```bash
# vendored files touched in current diff
git diff --name-only -- crates/tasklens-ui/src/dioxus_components crates/tasklens-ui/assets/dx-components-theme.css

# components that declare base classes
rg -n 'class:\s*"' crates/tasklens-ui/src/dioxus_components -g 'component.rs'

# components that explicitly merge attributes
rg -n 'merge_attributes\(' crates/tasklens-ui/src/dioxus_components -g 'component.rs'

# potential raw class overrides in RSX callsites (manual review required)
rg -nUP '(?s)\b[A-Z][A-Za-z0-9_]*\s*\{[^}]*\bclass\s*:' crates/tasklens-ui/src --glob '*.rs'
```

If noise is high, move checks into a lint script with an allowlist of approved
exceptions by path.
