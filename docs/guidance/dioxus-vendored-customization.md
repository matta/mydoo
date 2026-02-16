# Dioxus Vendored Customization Guidance

This guidance defines how `tasklens-ui` should customize vendored Dioxus
components in general (behavior, API shape, composition, accessibility, and
state ownership) while minimizing vendor drift.

Applies to:

- `crates/tasklens-ui/src/dioxus_components/**`
- app wrappers/adapters that consume vendored components
- feature code that composes vendored components

Goal: preserve vendored interaction semantics by default, centralize app
customization in app-owned code, and keep vendor upgrades routine.

## Decision Summary

1. Prefer vendored behavior and API contracts as the baseline.
2. Prefer app-owned wrappers/adapters over direct vendored edits.
3. Treat vendored patches as last resort and keep them minimal.
4. Require tests for every non-trivial behavior customization.
5. Track every vendor delta with a removal/source-project plan.

## Customization Ladder (Use In Order)

Before editing vendored component code, escalate in this order:

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
5. **Vendored patch (last resort)**
   - Only when vendored API/composition cannot represent required behavior.

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
   express it.

## Styling Is A Special Case

Styling customization has its own policy and guardrails.

Use: [`dioxus-vendored-styling.md`](./dioxus-vendored-styling.md)

In this document, styling should be treated as a delegated concern. Apply the
customization ladder here for behavior/API changes, and the styling ladder in
the styling guidance for visual changes.

## Vendored Patch Policy (Behavior)

A vendored behavior patch is acceptable only if all conditions hold:

1. The issue cannot be solved with vendored props, wrappers, or composition.
2. The patch is minimal, isolated, and documented as vendored delta.
3. Regression coverage is added at the right level (unit/integration/E2E).
4. A source-project issue/PR is opened for non-trivial changes when practical.
5. A follow-up item exists to remove or reconcile the local delta.

## Test Expectations

For non-trivial customization, add tests that verify observable behavior:

1. Interaction flows (keyboard/pointer/focus/open-close).
2. State ownership boundaries (controlled/uncontrolled usage).
3. Accessibility-critical behavior (roles/labels/disabled states).
4. Integration behavior in the target feature context.

Favor realistic user interactions over synthetic events in E2E tests.

## Review Checklist

1. Was the customization ladder followed before touching vendored files?
2. Does the change preserve vendored interaction and accessibility semantics?
3. Are controlled/uncontrolled patterns still correct?
4. Are callbacks and attributes forwarded safely?
5. If vendor files changed, is delta documented and tracked?
6. Is there test coverage for the customized behavior?
7. Is styling guidance applied via
   [`dioxus-vendored-styling.md`](./dioxus-vendored-styling.md) for visual
   changes?

## Optional Guardrails

Use these checks to spot risky customization changes:

```bash
# vendored behavior files touched in current diff
git diff --name-only -- crates/tasklens-ui/src/dioxus_components

# wrappers/adapters that may need review for callback passthrough
rg -n 'on[a-zA-Z]+:\s*move \|' crates/tasklens-ui/src --glob '*.rs'

# potential direct vendored usage in feature code (consider adapters first)
rg -n 'use crate::dioxus_components::|crate::dioxus_components::' crates/tasklens-ui/src --glob '*.rs'
```

If these are noisy, move them to a lint script with path-based allowlists.
