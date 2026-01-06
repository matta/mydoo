# Behavior Guidelines

- Always address the user with the salutation "Howdy".

# Development Guidelines

## Package Management

- Use `pnpm` for all package management and scripts.

## Environment Initialization

- **Dependency Installation:** Run `pnpm install`.
- **Clean Install:** If you need to clean the environment, use
  `scripts/aggressive-git-clean.sh` followed by `pnpm install`.

## Git Workflow

- **Clean Tree Rule:** Before starting unrelated work or a new development
  phase, run `git status`. If the working tree is not clean, STOP and notify the
  user.
- **Git Commit Rule:** NEVER commit a git change without an explicit command
  from the user beginning with `git commit`. If the user asks to commit (e.g.,
  "commit this") without the explicit command, STOP and ask for confirmation.
  - **Enforcement Protocol:** When work is complete and ready to commit, the
    Agent MUST explicitly state "Waiting for git commit command" and STOP. The
    Agent must NOT infer or assume permission to commit from context, prior
    commits, or phrases like "continue" or "finish this."
  - **Future Tense Prohibition:** Phrases like "we'll commit", "undo and redo",
    or "fix and commit" act as plans, NOT commands. You must execute the work
    (e.g., the undo/fix) and then **STOP** to request a fresh commit command.
  - **Verification Check:** Before executing any `git commit`, the Agent must
    verify: "Did the user's most recent message start with 'git commit'?" If NO,
    STOP immediately.
- **Git Push Rule:** NEVER run `git push` unless the user explicitly commands it
  (e.g., "push", "sync"). "Commit" does NOT imply "Push". You must stop after
  committing.
- **Ambiguity Protocol:** If a user's instruction combines a correction ("undo
  this") with a future action ("we'll commit"), treating the future action as
  authorized is a **Violation**. You must strictly separate the immediate
  correction from the subsequent authorization.
- **Git Presubmit Rule:** NEVER use `--no-verify`. On presubmit failure: fix
  trivial issues and retry; otherwise STOP AND WAIT.
- **Foreground Commit Rule:** ALWAYS run `git commit` in the foreground
  (synchronously). Presubmit hooks often fail or warn; immediate feedback is
  required to retry or fix issues promptly.
- **Verification Protocol:** The Agent must NEVER report a shell command as
  successful based on log output alone. The Agent MUST verify that the process
  `Exit Code` is `0` (via `command_status`) before claiming success.
- **Secret Verification:** The Agent must set the `AGENT_GIT_COMMIT_SECRET`
  environment variable to the correct value ("I am an AI agent and the user
  explicitly gave me permission to commit in the most recent prompt they issued,
  and I have recently read AGENTS.md and am following all the git commit
  requirements.") when running `git commit`.

## Documentation

- All new code must have documentation comments. Explain all non-obvious logic.
- Do not remove comments from existing code unless asked to do so by the user.
- Keep comments up to date.
- **Code Review Guidance:** See
  [docs/guidance/code-review.md](docs/guidance/code-review.md) for best
  practices on TypeScript, React, Redux, and testing conventions.
- **Markdown Style:** Use markdown bold and italics rarely.

## Testing Requirements

- We use Vitest for testing.
- All new code must have tests.
- **Strict Verification:** ALWAYS run `pnpm verify` before certifying a change.
  You MAY use targeted `turbo` commands during development, but you MUST run the
  full `pnpm verify` sequence before asking the user to commit.

### Test Commands

```bash
# All unit tests (monorepo-wide)
pnpm test

# All E2E tests (monorepo-wide)
pnpm test:e2e

# All unit tests in a specific package
pnpm exec turbo run test --filter <package>
# e.g. pnpm exec turbo run test --filter @mydoo/client

# Specific test file within a package
pnpm exec turbo run test --filter <package> -- <RelativePathToTestFile>
# e.g. pnpm exec turbo run test --filter @mydoo/client -- src/test/utils/date-formatter.test.ts

# Specific E2E feature or test with Playwright/BDD
# Note: Always run bddgen first if modifying .feature files
pnpm exec turbo run test-e2e --filter <package> -- --project=<project> -g <pattern>
# e.g. pnpm exec turbo run test-e2e --filter @mydoo/client -- --project='bdd-desktop' -g 'Due Dates'

# Fully build everything and re-run all tests including e2e (monorepo-wide), ignoring cache
TURBO_FORCE=true pnpm exec turbo run check-agent
```

## Context Convention: `ROLLING_CONTEXT.md`

For efforts spanning multiple sessions or commits, we maintain a root-level
`ROLLING_CONTEXT.md`.

- **Purpose:** Persistent, user-editable "working memory" for the project's
  current focus.
- **Workflow:**
  - **User:** Updates this file to set high-level goals, shift direction, or
    clarify requirements.
  - **Agent:** Reads at new phase start (per Clean Tree Rule). Must keep rolling
    task lists up to date. May update other agent-designated sections
    autonomously.
- **Contrast with System Artifacts:**
  - `task.md` / `implementation_plan.md`: The agent's _ephemeral, internal_
    checklist for the immediate next step.
  - `ROLLING_CONTEXT.md`: The _persistent, shared_ narrative of the broader
    effort. It is not tracked in git.

## Testing Strategy

### Core Philosophy: Fidelity First

We prioritize fidelity. Because this is a Local-First application relying on
**Automerge** (WASM, Binary CRDTs) and specific browser technologies
(`IndexedDB`, `TextEncoder`, `Crypto`), our strategic goal is to **avoid
simulated environments like JSDOM**.

### 1. The 3-Tier Architecture (Goal State)

| Tier       | Scope      | Target Infrastructure   | Rationale                                          |
| :--------- | :--------- | :---------------------- | :------------------------------------------------- |
| **Tier 1** | Pure Logic | **Node.js**             | Native WASM performance; no DOM pollution.         |
| **Tier 2** | Components | **Vitest Browser Mode** | Real `IndexedDB` and `TextEncoder` implementation. |
| **Tier 3** | Journeys   | **Playwright**          | Full multi-tab sync and offline/online simulation. |

### 2. Current Implementation Status

> **Note:** We are largely in **JSDOM** today. Use the rules below for daily
> development.

**Tier 1 (Packages/Logic):**

- **Status:** Runs in **Node**.
- **Command:** `pnpm test` (via Turbo).

**Tier 2 (Client Components):**

- **Status:** Runs in **JSDOM**.
- **Command:** `pnpm test` (via Turbo).
- **Aspiration:** We will migrate this to Browser Mode (`--project=browser`).

**Tier 3 (E2E):**

- **Status:** Runs in **Chromium**.
- **Command:** `pnpm test:e2e`.

### 3. AI Agent Instructions

1.  **New Test Mandate:** All **NEW** tests must strictly follow the
    **Executable Specs** style (Section 4). **Do not mimic legacy patterns**
    found in existing files.
2.  **Generate Compatible Tests:** Write component tests using
    `@testing-library/react` that pass in JSDOM.
3.  **Respect the Goal:** Avoid relying on JSDOM-specific APIs (`jest-dom`
    extensions are okay, but don't access `window.` internals directly if a
    standard API exists).
4.  **Mocking Strategy:**
    - **Tier 1:** No mocks. Test logic directly.
    - **Tier 2:** **Mock AutomergeRepo**. JSDOM struggles with the Repo's
      binary/WASM networking. Mock the handle to ensure component tests are
      stable in the simulated environment.
5.  **Prefer `userEvent` over `fireEvent`**: Always use
    `@testing-library/user-event`. Use of `fireEvent` from
    `@testing-library/react` is strictly prohibited as it doesn't simulate real
    browser interactions and often leads to flaky tests in async environments.

### 4. Executable Specs & Style Guide

Tests are **Executable Specifications**. They should read as high-level
narratives using the project's **Ubiquitous Language**.

**Ubiquitous Language Rule:** ALWAYS use domain terms (`Inbox`, `Plan`, `Do`,
`Balance`, `Context`). See `docs/design/prd.md` for the dictionary. Reject
implementation-level terms in test narratives.

- ✅ `plan.createTask("Buy Milk")`
- ❌ `button.click()`, `journal.entry()`

**Fluent Architecture (Playwright E2E):**

- Use `test.step` for high-level narrative structure.
- **NO** direct `page`, `locator`, or CSS selectors in the spec layer—delegate
  to Domain Helpers or Page Objects.
- Use **Inline Gherkin** comments (`// Given`, `// When`, `// Then`) to define
  intent.

**Example:**

```typescript
test('User can organize tasks', async ({plan}) => {
  await test.step('Create task', async () => {
    // Given the user is on the Plan view
    // When they create a new task
    await plan.createTask('Buy Milk');
    // Then the task appears in the list
  });
});
```

### 5. BDD Support (Playwright)

We use `playwright-bdd` to generate Playwright tests from Gherkin `.feature`
files.

1.  **Generation Step:** BDD tests MUST be generated manually before execution.
    Tests are NOT automatically generated by `pnpm test`.

    ```bash
    # Generate BDD tests for the client
    pnpm --filter @mydoo/client exec bddgen

    # Or manually from apps/client
    pnpm exec bddgen
    ```

2.  **Output Location:** Generated tests are stored in `tests/e2e/.features-gen`
    (split by profile, e.g., `/desktop`, `/mobile`). These files are ignored by
    git and Prettier.
3.  **Stale Files:** If you delete or rename a `.feature` file, you MUST run
    `bddgen` to remove the corresponding stale `.spec.js` file. Stale files will
    cause `test-e2e` failures.

## Testing Workflows

### Running Specific Tests

To run only the algorithm BDD tests:

```bash
pnpm test tests/unit/algorithm.test.ts
```

To run a specific feature within the algorithm tests, use the `FEATURE_FILTER`
environment variable:

```bash
FEATURE_FILTER="Inheritance" pnpm test tests/unit/algorithm.test.ts
```

Or use the standard Vitest `-t` flag (which runs all but skips non-matching):

```bash
pnpm test tests/unit/algorithm.test.ts -t "Inheritance"
```
