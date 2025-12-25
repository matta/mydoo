# Development Guidelines

## Package Management

- Use `pnpm` for all package management and scripts.

## Git Workflow

- **Clean Tree Rule:** Before starting unrelated work or a new development phase, run `git status`. If the working tree is not clean, STOP and notify the user.
- **Git Presubmit Rule:** NEVER use `--no-verify`. On presubmit failure: fix trivial issues and retry; otherwise STOP AND WAIT.
- **Foreground Commit Rule:** ALWAYS run `git commit` in the foreground (synchronously). Presubmit hooks often fail or warn; immediate feedback is required to retry or fix issues promptly.

## Documentation

- All new code must have documentation comments. Explain all non-obvious logic.

## Testing Requirements

- We use Vitest for testing.
- All new code must have tests.

## Context Convention: `ROLLING_CONTEXT.md`

For efforts spanning multiple sessions or commits, we maintain a root-level `ROLLING_CONTEXT.md`.

- **Purpose:** Persistent, user-editable "working memory" for the project's current focus.
- **Workflow:**
  - **User:** Updates this file to set high-level goals, shift direction, or clarify requirements.
  - **Agent:** Reads at new phase start (per Clean Tree Rule). Must keep rolling task lists up to date. May update other agent-designated sections autonomously.
- **Contrast with System Artifacts:**
  - `task.md` / `implementation_plan.md`: The agent's *ephemeral, internal* checklist for the immediate next step.
  - `ROLLING_CONTEXT.md`: The *persistent, shared* narrative of the broader effort.

## Hybrid AI-Assisted Testing Strategy

We use a risk-based testing protocol. Not everything is tested the same way.

### 1. The Core Protocol: "Stop & Plan"

**Rule:** The AI must Stop & Plan before:
- Adding new features or significantly changing existing behavior.
- Making UI changes that alter user-facing workflows or visual states.
- Modifying code that lacks test coverage (plan must include adding tests).

For trivial fixes (typos, formatting, obvious one-liners), the agent may proceed directly.

1.  **Categorize:** Declare the feature's complexity bucket (see below).
2.  **Plan:** Output a specification (Text Plan for simple features, Gherkin for complex ones).
3.  **Approve:** Validate the plan against edge cases.
4.  **Execute:** Write code only after explicit approval.

### 2. The Decision Matrix (4-Bucket System)

#### 1. Unit (**Pure Logic**)
- **Use When:** Logic runs without a browser (math, parsers, hooks).
- **Tooling:** `Vitest`
- **Spec:** Natural Language Comments

#### 2. Component (**Isolated UI**)
- **Use When:** Visual states (Loading, Error) or atomic interactions.
- **Tooling:** `Storybook`
- **Spec:** `.stories.tsx`

#### 3. Integration
- **Use When:** Routine flows, navigation, simple CRUD.
- **Tooling:** `Playwright`
- **Spec:** `// Spec: User can...` (Natural Language)

#### 4. System
- **Use When:** Data integrity, sync, auth, complex state machines, subtle differences between mobile and web, significant workflows. When uncertain between Bucket 3 and 4, ask the user for clarification.
- **Tooling:** `Playwright-BDD`
- **Spec:** `.feature` (Gherkin)

### 3. Implementation Rules

- **For Bucket 1 (Unit):** Follow existing conventions. Test public APIs; internal helpers need not be tested directly, unless complexity warrants it.
- **For Bucket 2 (UI):** Use "Portable Stories." Define states in Storybook, then import them into Unit/Interaction tests to avoid duplication.
- **For Bucket 3 (Integration):** Use **Inline Specs**. Tests must describe the requirement directly (`// Spec: Clicking X does Y`), avoiding IDs or external references.
- **For Bucket 4 (System):** Use **Centralized Gherkin**. The `.feature` file is the contract. AI must map steps to reusable Page Object methods.

### 4. Quick Select

- _Need to see it?_ -> **Storybook**
- _Need to calculate it?_ -> **Vitest**
- _Need to navigate it?_ -> **Playwright**
- _Is it complex or critical?_ -> **BDD (Gherkin)**
