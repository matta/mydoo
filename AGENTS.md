# Development Guidelines

## Package Management

- Use `pnpm` for all package management and scripts.

## Git Workflow

- **Clean Tree Rule:** Before starting unrelated work or a new development phase, run `git status`. If the working tree is not clean, STOP and notify the user.
- **Git Commit Rule:** NEVER commit a git change without an explicit command from the user beginning with `git commit`. If the user asks to commit (e.g., "commit this") without the explicit command, STOP and ask for confirmation.
- **Git Presubmit Rule:** NEVER use `--no-verify`. On presubmit failure: fix trivial issues and retry; otherwise STOP AND WAIT.
- **Foreground Commit Rule:** ALWAYS run `git commit` in the foreground (synchronously). Presubmit hooks often fail or warn; immediate feedback is required to retry or fix issues promptly.
- **Verification Protocol:** The Agent must NEVER report a shell command as successful based on log output alone. The Agent MUST verify that the process `Exit Code` is `0` (via `command_status`) before claiming success.

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
  - `task.md` / `implementation_plan.md`: The agent's _ephemeral, internal_ checklist for the immediate next step.
  - `ROLLING_CONTEXT.md`: The _persistent, shared_ narrative of the broader effort.

## Testing Strategy: Executable Specs

We use a risk-based testing protocol. Not everything is tested the same way.

We reject fragile, low-level DOM tests. We use **Fluent, Literate Architecture** where tests act as high-level executable specifications using the project's **Ubiquitous Language**.

### 1. The Core Protocol: "Stop & Plan"

**Rule:** The AI must Stop & Plan before:

- Adding new features or significantly changing existing behavior.
- Making UI changes that alter user-facing workflows.
- Modifying code that lacks test coverage.

**The Planning Step:**

1.  **Draft the Spec:** For complex features, the AI must draft the _Test Case_ (in TypeScript) before writing implementation code. This acts as the requirement definition.
2.  **Verify Language:** Ensure the draft uses domain terms (e.g., `plan.createTask()`, `do.complete()`), NOT generic or implementation terms (`journal.entry()`, `button.click()`).
3.  **Ubiquitous Language Rule:** ALWAYS use the specific Vocabulary of the Domain (e.g., `Inbox`, `Plan`, `Do`, `Balance`, `Context`). See `docs/design/prd.md` for the dictionary.

### 2. Architectural Layers

#### Layer 1: The Executable Spec (Test File)

- **Tooling:** `Playwright`
- **Style:** High-level narrative using `test.step`.
- **Constraint:** **NO** direct usage of `page`, `locator`, or CSS selectors allowed here.
- **Example:**
  ```typescript
  test("User can organize tasks", async ({ plan }) => {
    await test.step("Create task", async () => {
      await plan.createTask("Buy Milk");
    });
  });
  ```

### 3. Implementation Rules

- **For Bucket 1 (Unit):** Follow existing conventions. Test public APIs; internal helpers need not be tested directly, unless complexity warrants it.
- **For Bucket 2 (UI):** Use "Portable Stories." Define states in Storybook, then import them into Unit/Interaction tests to avoid duplication.
- **For Executable Specs (Playwright):** Use **Fluent Architecture**. Start with **Inline Gherkin** comments (`// Given/When/Then`) to define intent. Test bodies must be readable narratives. Map all steps to Domain Helpers or Page Objects.

### 4. Quick Select

- _Need to see it?_ -> **Storybook**
- _Need to calculate it?_ -> **Vitest**
- _Need to navigate it?_ -> **Playwright**
- _Is it complex or critical?_ -> **Executable Spec (Playwright)**
