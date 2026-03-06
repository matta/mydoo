# Dioxus Vendored Customization Guidance

This guidance defines how `tasklens-ui` should customize vendored Dioxus
components (behavior, API shape, composition, accessibility, and state
ownership).

Applies to:

- `crates/tasklens-ui/src/dioxus_components/**`
- app wrappers/adapters that consume vendored components
- feature code that composes vendored components

## Philosophy

The upstream dioxus-components project is designed to be forked and edited. It
does not provide exhaustive customization hooks — editing the source is the
expected extension mechanism. We should do so as well, when prudent.

Edit vendored code when it is the simplest correct solution. Avoid gratuitous
changes that create merge noise without clear product value.

Strong default: keep vendored code independent from app-owned modules.
Vendored files should not import `crate::app_components::*` or
`crate::views::*` unless there is an explicit, documented exception.

## Deliberate vs Gratuitous

The same distinction from the
**[Vendored Styling Guidance](./dioxus-vendored-styling.md#deliberate-vs-gratuitous)**
applies here. When weighing whether a vendored behavior edit is the right
approach, consider:

- Does the edit meaningfully simplify application code?
- Is the change surgical (additive or narrowly scoped, not restructuring)?
- Is merge-conflict risk low (stable area of the upstream codebase)?
- Would the alternative (wrapper, adapter, composition) create brittleness or
  disproportionate complexity?

If most answers are "yes", the vendored edit is likely the right call.

> [!CAUTION]
> **Do not:**
>
> - Reformat vendored code to match app conventions.
> - Rename vendored variables, types, or functions for consistency.
> - Make drive-by improvements to vendored files while working on unrelated
>   tasks.
> - Restructure vendored component internals unless the restructure _is_ the
>   task.

## Approaches (Prefer Simplicity)

When you need to change how a vendored component behaves, consider these
approaches and choose the one that produces the simplest, most maintainable
result:

1. **Use supported vendored props and composition points**
   - Prefer documented primitive/component props (`value`, `open`, callbacks,
     etc.) before any local structural changes.
2. **App wrapper/adapter**
   - Build app-owned wrappers that encode domain defaults, normalize props, and
     centralize repeated behavior.
3. **App-owned composition**
   - Add behavior around the component (container logic, orchestration,
     validation, data mapping) without forking internals.
4. **Scoped behavior override**
   - Use targeted wrapper logic for one-off product constraints, then promote to
     reusable adapter if it repeats.
5. **Direct vendored edit**
   - When approaches 1–4 would create brittleness or disproportionate
     complexity, edit the vendored source directly.
   - Keep the edit surgical. Document rationale in the commit message.
   - Keep vendored internals self-contained. For shared look/feel, align
     vendored code to shared design tokens instead of importing app-owned
     components.

These are not a strict escalation ladder — sometimes approach 5 is simpler and
more correct than approach 2. Use judgment.

## Behavioral Contract For Vendored Components

When customizing behavior, preserve these invariants unless there is an explicit
product decision to diverge:

1. **Interaction model**
   - Keep keyboard, pointer, and focus behavior compatible with vendored
     behavior.
2. **Accessibility semantics**
   - Preserve ARIA roles/states, labels, and form relationships.
3. **Controlled/uncontrolled semantics**
   - Do not mix ownership models accidentally (`value` vs `default_value`,
     `open` vs `default_open`).
4. **Event pipeline**
   - Wrappers must not swallow vendored or consumer callbacks; chain handlers in
     a predictable order.
5. **Attribute passthrough**
   - Preserve `id`, `name`, `aria-*`, `data-*`, and required form attributes.

## Adapter-First Pattern (Recommended)

Use app-owned adapters (for example, `AppSelect`, `AppDialog`, `AppInput`) as
the primary extension point.

Adapter responsibilities:

1. Map domain semantics to vendored props.
2. Set stable app defaults for behavior.
3. Normalize callback signatures and side effects.
4. Keep callsites simple and consistent.

Adapter anti-responsibilities:

1. Re-implement vendored primitive behavior without a strong reason.
2. Embed large per-feature conditionals that belong in feature code.
3. Introduce app-specific behavior by editing vendor files when wrappers can
   express it just as simply.
4. Add app-owned module dependencies inside vendored code when token alignment,
   wrappers, or composition can solve the need.

## Exception Rubric (Soft Rule)

Vendored-to-app dependencies are discouraged, not absolutely forbidden. Approve
an exception only when all are true:

1. The need is product-critical and near-term.
2. Token alignment, wrappers, and vendored-local edits were considered and are
   materially worse.
3. The exception scope is narrow and reversible.
4. The rationale is documented in the commit message and allowlist.

## Styling Is A Special Case

Styling customization has its own policy and guardrails.

Use:

- **[Dioxus Vendored Styling Guidance](./dioxus-vendored-styling.md)** for rules on overriding vendored component CSS.
- **[CSS Style Guide](./css.md)** for rules on writing new app-level CSS tokens, layouts, and touch targets.
- See **[Two Kinds of Rules](./css.md#two-kinds-of-rules)** for which CSS rules apply to vendored code.

In this document, styling should be treated as a delegated concern. Apply the
approaches here for behavior/API changes, and the styling guidance for visual
changes.

## Test Expectations

For non-trivial customization, add tests that verify observable behavior:

1. Interaction flows (keyboard/pointer/focus/open-close).
2. State ownership boundaries (controlled/uncontrolled usage).
3. Accessibility-critical behavior (roles/labels/disabled states).
4. Integration behavior in the target feature context.

Favor realistic user interactions over synthetic events in E2E tests.

## Review Checklist

1. Is the vendored edit deliberate and product-driven, not gratuitous?
2. Is the change surgical and narrowly scoped?
3. Does the change preserve vendored interaction and accessibility semantics?
4. Are controlled/uncontrolled patterns still correct?
5. Are callbacks and attributes forwarded safely?
6. Is there test coverage for non-trivial customized behavior?
7. Is styling guidance applied via
   [`dioxus-vendored-styling.md`](./dioxus-vendored-styling.md) for visual
   changes?
8. Are vendored-to-app dependencies avoided unless explicitly approved under
   the exception rubric?

## Optional Guardrails

Use these checks to spot risky customization changes:

```bash
# enforce vendored ownership boundaries
cargo xtask check-vendored-boundaries

# vendored behavior files touched in current diff
git diff --name-only -- crates/tasklens-ui/src/dioxus_components

# wrappers/adapters that may need review for callback passthrough
rg -n 'on[a-zA-Z]+:\s*move \|' crates/tasklens-ui/src --glob '*.rs'

# potential direct vendored usage in feature code (consider adapters first)
rg -n 'use crate::dioxus_components::|crate::dioxus_components::' crates/tasklens-ui/src --glob '*.rs'
```

If these are noisy, move them to a lint script with path-based allowlists.
