# Just Task Mapping & Dependency Philosophy

This document defines the mapping from legacy Turborepo tasks to Just recipes and establishes the project's philosophy regarding task dependencies.

## Dependency Philosophy: Velocity First

To maximize development velocity, Just recipes for `test` and `build` should only invoke tasks that are **strictly required** for them to function.

1.  **Development Rules (`test`, `build`):**
    - Do NOT depend on linting or style checks (`check-style`, `check-biome`).
    - Do NOT depend on non-essential validation (`check-types`) unless necessary for the build artifact itself.
    - The goal is to allow developers to iterate quickly on logic without waiting for unrelated static analysis.
2.  **Verification Rules (`verify`, `check-agent`):**
    - MUST include all validation gates.
    - `verify` is the source of truth for "ready to commit" state.

## Task Mapping

| Turbo Task                | Just Recipe            | Dependencies (Velocity First)      | Description                          |
| :------------------------ | :--------------------- | :--------------------------------- | :----------------------------------- |
| `//#check-format-root`    | `check-format-root`    | None                               | Prettier check on root files         |
| `//#fix-format-root`      | `fix-format-root`      | None                               | Prettier fix on root files           |
| `//#check-biome-root`     | `check-biome-root`     | None                               | Biome check on root                  |
| `//#fix-biome-root`       | `fix-biome-root`       | None                               | Biome fix on root                    |
| `//#check-syncpack-root`  | `check-syncpack-root`  | None                               | Syncpack check                       |
| `//#fix-syncpack-root`    | `fix-syncpack-root`    | None                               | Syncpack fix                         |
| `//#check-catalog-root`   | `check-catalog-root`   | None                               | Knip-based catalog check             |
| `//#check-deps-root`      | `check-deps-root`      | None                               | Knip check on root                   |
| `//#check-filenames-root` | `check-filenames-root` | None                               | Custom filename linting              |
| `//#check-eslint-root`    | `check-eslint-root`    | None                               | ESLint on root                       |
| `//#fix-eslint-root`      | `fix-eslint-root`      | None                               | ESLint fix on root                   |
| `check-eslint`            | `check-eslint`         | None                               | ESLint on all packages               |
| `check-format`            | `check-format`         | None                               | Prettier check on all packages       |
| `check-style`             | `check-style`          | _Aggregate_                        | Aggregated style check               |
| `check-types`             | `check-types`          | None                               | TypeScript type check                |
| `check-rust`              | `check-rust`           | None                               | Clippy and fmt check                 |
| `check-human`             | `check-human`          | _Aggregate_                        | Full validation before human review  |
| `test`                    | `test`                 | **None**                           | Unit tests (Fast)                    |
| `test-e2e`                | `test-e2e`             | `build-ui`                         | End-to-end tests (Requires UI build) |
| `check-agent`             | `check-agent`          | `check-human`, `test`, `test-e2e`  | Full CI-like validation              |
| `build`                   | `build`                | **None**                           | Build all packages (Fast)            |
| `fix`                     | `fix`                  | `fix-style`, `fix-syncpack-root`   | Run all auto-fixes                   |
| `verify`                  | `verify`               | `check-rust`, `fix`, `check-agent` | The ultimate verification command    |
| `dev`                     | `dev`                  | None                               | Development server                   |
