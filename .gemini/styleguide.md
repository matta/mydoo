# Repository review guide for Gemini Code Assist

## Scope to treat as merge-sensitive

- Vendored component sources in `/Users/matt/src/mydoo/crates/tasklens-ui/src/dioxus_components/**`
- Tasklens UI integration code that depends on `dioxus_primitives` (files with `use dioxus_primitives::...`)

These areas are updated from upstream and are intentionally kept close to upstream layout and behavior.

## Change policy for `dioxus_primitives` and vendored component work

- Prefer the smallest possible patch that fixes the concrete issue.
- Avoid broad refactors, API redesign, renames, and file moves unless they are required for correctness or build/test failure.
- Do not propose style-only cleanups (formatting churn, naming tweaks, import reordering, comment rewrites, or "consistency" edits) in these areas.
- Do not suggest opportunistic improvements that are unrelated to the reported bug/task.
- Keep structure and call patterns aligned with upstream unless a local constraint requires divergence.

## Review priorities

When reviewing changes in these areas, prioritize:

1. Correctness bugs and regressions
2. Compile/runtime/test failures
3. Accessibility/behavior defects
4. Dependency pinning and lockfile consistency

De-prioritize stylistic preferences unless they block correctness.

## PR guidance

- If a proposed change in these files is not strictly necessary, recommend dropping it.
- If a broader cleanup is truly needed, recommend doing it in a separate PR with explicit justification.
