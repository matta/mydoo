# Build System Design

This document outlines the design and philosophy of the build system for the `mydoo` repository. We use [Moonrepo](https://moonrepo.dev/) as our primary build tool and task runner, with `package.json` scripts serving as convenient entry points.

## Philosophy: "Moon First"

Our build system follows a "Moon First" philosophy. This means that **all significant build, test, and quality assurance tasks are defined and managed by Moon**, not by ad-hoc shell scripts or direct CLI calls in `package.json`.

### Why Moon?

- **Caching:** Moon caches outputs (build artifacts, test results) based on inputs. If inputs haven't changed, the task is skipped.
- **Dependency Graph:** Moon understands the dependency graph between projects (e.g., `client` depends on `tasklens`).
- **Parallelization:** Moon automatically runs independent tasks in parallel.
- **Environment Consistency:** Moon ensures tasks run in a consistent environment.

## `package.json` Script Criteria

The `package.json` `scripts` block serves as a developer-facing API (aliases) for the underlying Moon tasks.

### 1. When to Invoke Moon (`moon run`)

A script **MUST** invoke `moon run` if:

- **It produces artifacts:** e.g., `build` (generates `dist/`).
- **It is computationally expensive:** e.g., `test`, `typecheck`.
- **It depends on other projects:** e.g., `client:build` needing `tasklens:build`.
- **It provides quality assurance:** e.g., `lint`, `format`, `check`.

**Example:**

```json
"scripts": {
  "build": "moon run client:build",
  "test": "moon run client:test"
}
```

### 2. When NOT to Invoke Moon

A script **MAY** bypass Moon if:

- **It is a local-only dev server:** e.g., `vite` (though Moon can wrap this, direct invocation is sometimes preferred for interactive TTY preservation).
- **It is a simple file operation:** e.g., `clean`: `rm -rf dist`.
- **It is a lifecycle hook:** e.g., `prepare`: `husky`.

### 3. Root vs. Project Tasks

- **Repo-wide Tools (Biome, ESLint):** These tasks are defined in the globally available `moon.yml` (or root `moon.yml`) and should be invoked as `moon run root:<task>` or via a project-specific alias that depends on the root task.
- **Project-specific Tools (Vite, Vitest):** These are defined in the project's `moon.yml`.

## Task Naming

All tasks (both Moon tasks and `package.json` scripts) must adhere to the [Task Naming Specification](../guidance/task-naming.md).

## Reference Architecture

### Root configuration

- `moon.yml`: Defines repo-wide tasks like `root:lint`, `root:format`.
- `.moon/tasks.yml`: Defines shared tasks inherited by all projects (currently empty/minimal to enforce explicit root delegation where appropriate).

### Project configuration (`apps/client/package.json`)

```json
{
  "scripts": {
    "build": "moon run client:build", // Delegates to Moon
    "check": "moon run client:check", // Delegates to Moon
    "dev": "vite" // Direct execution (optional exception)
  }
}
```
