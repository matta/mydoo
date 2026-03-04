---
name: checklist-tasks
description: Write, review, and rewrite checklist bullets so each checkbox is a single actionable task that stays semantically true after completion. Use when converting findings, plans, or review notes into implementation checklists, especially when items are vague, diagnostic, or become contradictory when checked.
---

# Checklist Tasks

## Overview

Use this skill to produce checklist items that are directly executable and unambiguous. Every checkbox must read as an undone task when unchecked and a completed fact when checked.

## Checklist Item Contract

Write each item so it satisfies all rules:

1. Express one concrete action with an explicit verb.
2. Name the target artifact or location (`file`, `module`, `route`, `test`, `doc`).
3. State the expected observable outcome.
4. Include objective evidence of completion (where to verify).
5. Keep scope singular; split any item with multiple independent actions.
6. Ensure checked-state truth: after marking `[x]`, the sentence must remain semantically correct.

## Required Shape

Use this template:

```markdown
- [ ] <Action verb> <target> <context/constraint> so that <observable outcome>; verify via <test/doc/file>.
```

Accept equivalent wording if all required fields remain explicit.

## Rewrite Workflow

Apply this sequence when converting weak checklist bullets:

1. Classify source bullet as one of: `problem statement`, `risk statement`, `decision note`, or `task`.
2. If source is not a task, convert it into at least one action item and one validation item.
3. Replace vague verbs (`handle`, `fix`, `improve`, `clean up`) with concrete verbs (`add`, `remove`, `rename`, `route`, `assert`, `document`).
4. Add concrete scope (`which file`, `which route`, `which component`, `which spec`).
5. Add verifiable completion proof (`specific test`, `specific doc section`, `specific command outcome`).
6. Split compound bullets connected by `and`, `or`, or comma-separated independent actions.
7. Run the Quality Gate below; rewrite until all checks pass.

## Quality Gate (Pass/Fail)

Reject an item if any answer is `No`:

- Does the item start with a clear action verb?
- Does the item identify exactly what gets changed?
- Does the item define how success is observed?
- Does the item include where proof of completion lives?
- Would the sentence still be correct after replacing `[ ]` with `[x]`?
- Can one person complete it in one contiguous unit of work?

## Anti-Patterns to Eliminate

Rewrite these patterns immediately:

- Diagnostic phrasing: `- [ ] Back navigation is inconsistent`
- Missing-state phrasing: `- [ ] Return route is missing`
- Ambiguous intent: `- [ ] Improve route behavior`
- Multi-task bundles: `- [ ] Update route logic and tests and docs`
- Investigation-only placeholders without output artifact

## Before/After Examples

Example 1:

```markdown
# Bad

- [ ] Back navigation is inconsistent in settings flow.

# Good

- [ ] Update settings close action in `app_router.rs` to navigate to the recorded return route when present so back returns to the originating screen; verify via `settings-route-return.spec.ts`.
- [ ] Add E2E coverage in `settings-route-return.spec.ts` for `/plan`, `/do`, and `/inbox` entry paths to assert close returns to origin.
```

Example 2:

```markdown
# Bad

- [ ] Improve checklist wording.

# Good

- [ ] Rewrite all unchecked items in `findings.md` to begin with an action verb, name an artifact, and include explicit verification text.
- [ ] Add a short "Checklist Item Contract" section to `findings.md` describing the required item format so future additions follow the same rule.
```

## Output Requirements

When asked to revise a checklist:

1. Return only checklist items unless the user asks for commentary.
2. Preserve existing section order unless reordering is requested.
3. Do not mark any item complete unless evidence in current files proves completion.
4. Prefer many small, independent checkboxes over broad umbrella items.
