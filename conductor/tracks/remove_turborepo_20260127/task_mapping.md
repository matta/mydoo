# Turborepo to Just Task Mapping

| Turbo Task | Just Recipe | Dependencies | Description |
| :--- | :--- | :--- | :--- |
| `//#check-format-root` | `check-format-root` | None | Prettier check on root files |
| `//#fix-format-root` | `fix-format-root` | None | Prettier fix on root files |
| `//#check-biome-root` | `check-biome-root` | None | Biome check on root |
| `//#fix-biome-root` | `fix-biome-root` | None | Biome fix on root |
| `//#check-syncpack-root` | `check-syncpack-root` | None | Syncpack check |
| `//#fix-syncpack-root` | `fix-syncpack-root` | None | Syncpack fix |
| `//#check-catalog-root` | `check-catalog-root` | None | Knip-based catalog check |
| `//#check-deps-root` | `check-deps-root` | None | Knip check on root |
| `//#check-filenames-root` | `check-filenames-root` | None | Custom filename linting |
| `//#check-eslint-root` | `check-eslint-root` | None | ESLint on root |
| `//#fix-eslint-root` | `fix-eslint-root` | None | ESLint fix on root |
| `check-eslint` | `check-eslint` | None | ESLint on all packages |
| `check-format` | `check-format` | None | Prettier check on all packages |
| `check-style` | `check-style` | `check-eslint`, `check-eslint-root`, `check-format`, `check-biome-root`, `check-filenames-root`, `check-format-root` | Aggregated style check |
| `check-types` | `check-types` | None | TypeScript type check |
| `check-rust` | `check-rust` | None | Clippy and fmt check |
| `check-human` | `check-human` | `check-catalog-root`, `check-deps-root`, `check-syncpack-root`, `check-style`, `check-types` | Full validation before human review |
| `test` | `test` | `check-style`, `build`, `check-types` | Unit tests |
| `test-e2e` | `test-e2e` | `test` | End-to-end tests |
| `check-agent` | `check-agent` | `check-human`, `test`, `test-e2e` | Full CI-like validation |
| `build` | `build` | `check-style` | Build all packages |
| `fix` | `fix` | `fix-style`, `fix-syncpack-root` | Run all auto-fixes |
| `verify` | `verify` | `check-rust`, `fix`, `check-agent` | The "ultimate" verification command |
| `generate` | (Removed) | - | Dead task |
| `dev` | `dev` | None | Development server |

## Root package.json Scripts to Update
All scripts in the root `package.json` that currently run `turbo run <task>` will be updated to run `just <recipe>`.
- `build`, `check-agent`, `check-biome`, `check-eslint`, `check-human`, `check-style`, `check-types`, `check`, `dev`, `fix`, `test-e2e`, `test`.
- `verify` (shell script currently, will be moved to `just`).
