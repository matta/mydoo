# Research: Moonrepo Migration

**Feature**: `003-moonrepo-migration`
**Status**: In Progress

## Research Tasks

1.  **Task Migration**: Map every task in `turbo.json` to its equivalent `moon` configuration.
    -   *Input*: `turbo.json` "tasks" object.
    -   *Output*: `.moon/tasks.yml` (global) and project `moon.yml` (local).
2.  **CI Integration**: Determine the exact `moonrepo/setup-moon` configuration for GitHub Actions.
    -   *Goal*: Replace `actions/setup-node`, `pnpm/action-setup`, and `actions/cache` (for turbo) with moon's managed solution where possible, or integrate with them.
3.  **Toolchain Config**: Define the `.moon/toolchain.yml` settings to match the current `package.json` engines (Node 24) and `packageManager` (pnpm 10.26.1).

## Findings & Decisions

### 1. Task Migration Strategy

**Decision**: Use `.moon/tasks.yml` for globally consistent tasks (lint, format) and implicit task inheritance.

*Rationale*: Moon encourages inheriting tasks from the workspace root to enforce consistency, whereas Turbo often relies on root-level pipeline definitions.

**Mapping Table**:

| Turbo Task | Moon Equivalent | Type |
| :--- | :--- | :--- |
| `build` | `build` | Project Task (inputs: src, outputs: dist) |
| `test` | `test` | Project Task |
| `check-types` | `typecheck` | Global Task (inherited) |
| `check-biome` | `lint` | Global Task (inherited) |
| `//#check-deps` | `root:check-deps` | Root-level task (run with `moon run //:check-deps`) |

### 2. CI Integration

**Decision**: Use `moonrepo/setup-moon-action` in `.github/workflows/ci.yml`.

*Rationale*: This official action handles binary installation and toolchain hydration.

**Configuration Pattern**:

```yaml
- uses: moonrepo/setup-moon-action@v1
  with:
    # Auto-install node/pnpm based on toolchain.yml
    auto-install: true 
- run: moon ci
```

### 3. Toolchain Configuration

**Decision**: Explicitly configure `node` and `pnpm` in `.moon/toolchain.yml`.

*Rationale*: Matches the Spec requirement for strict version management.

**Draft Config**:

```yaml
node:
  version: '24.0.0' # Matches package.json engines
  packageManager: 'pnpm'
  pnpm:
    version: '10.26.1' # Matches package.json packageManager
```
