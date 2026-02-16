# Dioxus Upstream Styling Guidance

This guidance defines how `tasklens-ui` should use vendored Dioxus components
and `dioxus-primitives` while staying close to upstream design language and
still allowing targeted customization.

It is based on:

- post-facto review of commit `42fa299`
- current `tasklens-ui` CSS structure
- project goals for "upstream fidelity first, rare and deliberate exceptions"

## Decision Summary

1. Default to upstream visual fidelity.
2. Keep vendored code pristine by default.
3. Allow targeted local patches only when wrappers/tokens cannot solve the need.
4. Prefer app-owned adapter components for customization.
5. Keep `dx-components-theme.css` pristine in normal flow; exceptions are allowed
   but should be rare, documented, and reviewed explicitly.

## Current CSS Ownership Model

### Upstream-owned

- `crates/tasklens-ui/src/dioxus_components/**`
- `crates/tasklens-ui/assets/dx-components-theme.css`

### App-owned

- `crates/tasklens-ui/assets/app.css`
- `crates/tasklens-ui/src/app_components/**/*.css`
- `crates/tasklens-ui/src/views/**/*.css`

## Policy: Pristine vs Patch Tradeoff

Use this as the default rubric when deciding whether to edit vendored files.

| Mode                         | When to use                                     | What you gain                                | What you lose                             |
| ---------------------------- | ----------------------------------------------- | -------------------------------------------- | ----------------------------------------- |
| **A: Pristine**              | Default path for most changes                   | Lowest upgrade friction, clear upstream diff | Harder local customization                |
| **B: Small patch queue**     | Narrow blockers not solvable by wrappers/tokens | Practical fixes with controlled drift        | Ongoing rebase/merge overhead             |
| **C: Pragmatic local edits** | Temporary emergency only                        | Fast local iteration                         | High drift and long-term maintenance risk |

Project default is **A with selective B**, case by case.

## Customization Ladder (Use In Order)

Before editing vendored code, escalate through these steps in sequence:

1. **Theme and semantic tokens**
   - Prefer changing app semantic tokens in `assets/app.css`.
   - Keep `dx-components-theme.css` pristine unless a global theme defect cannot
     be expressed in app-owned tokens.
2. **Adapter or wrapper component**
   - Add app-owned wrappers around vendored components.
   - Use CSS modules for layout/context-specific styles.
   - Keep style policy and API decisions centralized.
3. **Scoped app override**
   - If needed, use narrowly scoped selectors in app-owned styles.
   - Avoid global broad selectors that can impact unrelated components.
4. **Vendor patch (last resort)**
   - Keep patch minimal and isolated.
   - Open upstream PR in parallel when practical.
   - Track and remove local patch when upstream lands.

## Adapter-First Pattern (Recommended)

Adopters should prefer app-owned adapters, for example:

- `AppInput`
- `AppTextarea`
- optional variant wrappers (`AppInputMono`, `AppInputLg`)

Adapter goals:

1. Preserve upstream base class/style behavior.
2. Centralize common width/layout behavior (avoid repeated inline styles).
3. Expose a small app-level API for recurring patterns.
4. Keep callsites free of per-instance styling workarounds.

## Known Input/Textarea Class Hazard

Current `Input` and `Textarea` implementations set a base class (for example,
`class: "input"`) and then spread `..attributes`. If a callsite passes `class:`,
the callsite class replaces the base class instead of merging with it.

### Why this matters

1. Raw `Input { class: ... }` can drop upstream border/padding/focus styles.
2. The same risk exists for raw `Textarea { class: ... }`.
3. This can silently violate the "upstream fidelity first" goal.

### Safe patterns (in priority order)

1. **Preferred for repeated usage: app-owned adapter**
   - Keep vendored component usage stable.
   - Centralize width/layout/typography policy in one place.
2. **Scoped CSS-module override via wrapper + `:global(...)`**
   - Use a local wrapper class and target upstream class as global.
3. **One-off inline style**
   - Acceptable as a temporary bridge, but avoid repeating at many callsites.

**Anti-pattern:**

```rust
// Replaces upstream base class.
Input { class: "w-full" }
```

**Recommended (scoped CSS module):**

```rust
div {
    class: Styles::search_container,
    Input { placeholder: "Search..." }
}
```

```css
.search_container :global(.input) {
  width: 100%;
}
```

Note: in Dioxus CSS modules, class selectors are scoped by default. Use
`:global(.input)` (or move the rule to a global stylesheet) when targeting a
vendored global class.

## Constraints For Theme File Edits

Goal state: keep `crates/tasklens-ui/assets/dx-components-theme.css` pristine.

A controlled edit may be acceptable only if:

1. The need is global and cannot be reasonably solved in app-owned CSS.
2. The change improves fidelity with intended upstream usage rather than
   creating app-specific divergence.
3. The edit is documented in the PR under "upstream delta."
4. A follow-up issue exists to reconcile with upstream (or revert).

## PR Review Checklist For Styling Changes

1. Does the change preserve upstream defaults where possible?
2. Could this be solved at a lower rung of the customization ladder?
3. Are new overrides scoped and minimal?
4. Did we avoid broad global selectors and per-callsite inline duplication?
5. If vendor edits exist, are they documented and tracked?
6. Was an upstream PR or issue opened for non-trivial local vendor patches?

## Practical Rules For `tasklens-ui`

1. Keep vendored component files in `src/dioxus_components` as close to
   upstream as practical.
2. Put app-specific layout and semantics in app-owned CSS modules first.
3. Keep shared app utility surface in `assets/app.css` small and intentional.
4. Prefer adapters over direct callsite workarounds.
5. Treat upstream-fidelity exceptions as rare decisions requiring explicit
   reviewer sign-off.

## Optional Guardrails

Add CI/script checks to catch common regressions:

```bash
# direct class usage on raw Input/Textarea callsites
rg -nUP '(?s)\bInput\s*\{[^}]*\bclass\s*:' crates/tasklens-ui/src --glob '*.rs'
rg -nUP '(?s)\bTextarea\s*\{[^}]*\bclass\s*:' crates/tasklens-ui/src --glob '*.rs'
```

If these checks are noisy, migrate them to a lint script that allows specific
approved exceptions by path.
