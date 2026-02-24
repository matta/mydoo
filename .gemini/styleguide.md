# Repository review guide for Gemini Code Assist

## Scope to treat as merge-sensitive

- Vendored component sources in `crates/tasklens-ui/src/dioxus_components/**`

These areas are updated from upstream and are intentionally kept close to upstream layout and behavior.

### Change policy for `crates/tasklens-ui/src/dioxus_components/**`

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

## Core Guidelines for aria-label

When authoring an aria-label, adhere to the following principles based on WAI-ARIA (Web Accessibility Initiative - Accessible Rich Internet Applications) specifications:

1. **Be Concise**: State the primary purpose or destination of the element as briefly as possible. "Edit task {task.title}" is clear and direct.
2. **Omit Interaction Instructions**: Do not include phrases like "click here," "press enter," or "double tap." Leave the interaction mechanics to the assistive technology.
3. **Omit the Role**: Do not include words like "button," "link," or "menu" in the label. The screen reader derives the role from the HTML element itself (e.g., `<button>`) or its role attribute. If you write `aria-label="Edit button"`, the screen reader will announce "Edit button, button."
4. **Use Only When Necessary**: An `aria-label` overrides the visible text content of an element for assistive technologies. It should primarily be used when an interactive element has no visible text (e.g., an icon-only button) or when the visible text does not provide enough context on its own (e.g., a "Read More" link that needs to become "Read More about {Article Title}"). If the element already contains visible, descriptive text, an `aria-label` is usually redundant.
5. **Capitalization and Punctuation**: Treat the label like a standard sentence or phrase. Capitalize the first letter, but avoid terminal punctuation (like a period) unless the label consists of multiple distinct sentences, as a period causes the screen reader to pause.
