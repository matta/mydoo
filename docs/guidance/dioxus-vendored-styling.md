# Dioxus Vendored Styling Guidance

This guidance defines styling policy for **all vendored Dioxus components** in
`tasklens-ui`.

Applies to:

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`
- app code that consumes vendored components

Goal: preserve vendored fidelity by default, while still allowing deliberate
product customization with minimal upgrade friction.

## Decision Summary

1. Treat vendored component code and vendored theme assets as pristine by
   default.
2. Put app-specific visual semantics in app-owned wrappers and CSS first.
3. Assume raw vendored components are not safe customization points unless
   explicitly documented.
4. Escalate to vendored patches only when wrapper/token approaches are
   insufficient.
5. Track every vendored delta so it can be removed or reconciled later.

## Ownership Boundaries

### Vendored-owned (keep pristine in normal flow)

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`

### App-owned (preferred customization surface)

- `crates/tasklens-ui/assets/app.css`
- `crates/tasklens-ui/src/app_components/**/*.css`
- `crates/tasklens-ui/src/views/**/*.css`
- app-level wrapper/adaptor components around vendored primitives

## Pristine vs Patch Rubric

| Mode                         | When to use                                     | What you gain                              | What you lose                             |
| ---------------------------- | ----------------------------------------------- | ------------------------------------------ | ----------------------------------------- |
| **A: Pristine**              | Default for most styling work                   | Lowest upgrade friction, clean vendor diff | Fewer direct hooks in vendored code       |
| **B: Small patch queue**     | Narrow blockers not solvable by wrappers/tokens | Practical fixes with controlled drift      | Ongoing merge/review overhead             |
| **C: Pragmatic local edits** | Temporary emergency only                        | Fast local iteration                       | High drift and long-term maintenance risk |

Project default is **A with selective B**.

## Customization Ladder (Use In Order)

Before editing vendored files, escalate in this order:

1. **App tokens and semantic CSS**
   - Prefer app semantic tokens and app-owned styles.
   - Keep global overrides small and intentional.
2. **App wrapper/adaptor component**
   - Encapsulate layout, spacing, variants, and repeated policy in app-owned
     wrappers.
   - Keep callsites free of one-off styling hacks.
3. **Scoped override from app code**
   - Use local wrapper classes and CSS modules.
   - Target vendored global classes with `:global(...)` when needed.
4. **Vendored patch (last resort)**
   - Keep patch minimal and isolated.
   - Open source-project issue/PR when practical.
   - Track removal criteria in the PR description and follow-up issue.

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

Goal state: keep `crates/tasklens-ui/assets/dx-components-theme.css` pristine.

A controlled edit is acceptable only if all conditions hold:

1. The need is truly global and cannot be solved cleanly in app-owned CSS.
2. The change aligns with vendored semantics rather than app-only divergence.
3. The PR documents the exact vendored delta.
4. A follow-up item exists to reconcile at the source project or remove the
   patch.

## Review Checklist (All Vendored Components)

1. Does the change keep vendored defaults intact wherever possible?
2. Was the customization ladder followed before touching vendored files?
3. Are overrides narrowly scoped and reversible?
4. Did we avoid broad global selectors and repeated per-callsite hacks?
5. If vendored files changed, is the delta documented and tracked?
6. Is there a source-project issue/PR for non-trivial local patches?
7. Does the change remain robust after a future `cargo xtask dx-components vendor` run?

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
