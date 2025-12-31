# Data Model: Moonrepo Configuration

**Feature**: `003-moonrepo-migration`

> **Note**: For this migration, "Entities" refer to the configuration structures required by Moonrepo.

## Configuration Entities

### 1. Workspace (`.moon/workspace.yml`)

Defines the project graph and structure.

| Field | Value/Rule | Rationale |
| :--- | :--- | :--- |
| `projects` | `['apps/*', 'packages/*']` | Auto-discovery of pnpm workspace packages. |
| `vcs.manager` | `'git'` | Use git for hashing and touched-files detection. |
| `vcs.defaultBranch` | `'main'` | Main branch for comparison. |

### 2. Toolchain (`.moon/toolchain.yml`)

Defines the managed environment.

| Field | Value/Rule | Rationale |
| :--- | :--- | :--- |
| `node.version` | `24` | Match current `package.json` engine. |
| `node.packageManager` | `pnpm` | Match current project standard. |
| `node.syncProjectWorkspaceDependencies` | `true` | Ensure `package.json` dependencies match workspace graph. |

### 3. Global Tasks (`.moon/tasks.yml`)

Defines tasks inherited by all projects (unless overridden).

| Task | Command | Inputs |
| :--- | :--- | :--- |
| `lint` | `biome check .` | `['@group(sources)', 'biome.json']` |
| `format` | `biome format .` | `['@group(sources)']` |
| `typecheck` | `tsc --noEmit` | `['@group(sources)', 'tsconfig.json']` |

### 4. Root Tasks (`package.json` / `root-project`)

Tasks that run at the repo root, mapped from Turbo's `//#` tasks.

| Task | Command | Migration Note |
| :--- | :--- | :--- |
| `check-deps` | `knip` | Run as a root-level moon task. |
| `check-filenames` | `./scripts/check-filenames.sh` | Run as a root-level moon task. |
