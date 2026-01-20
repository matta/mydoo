# Research Findings: ESLint Configuration

**Feature Branch**: `001-add-eslint-config`
**Date**: 2025-12-30

## 1. ESLint Plugin Compatibility

- **Decision**: Use `eslint-plugin-import-x` instead of `eslint-plugin-import`.
- **Rationale**: `eslint-plugin-import` has known compatibility issues with ESLint v9 flat config. `eslint-plugin-import-x` is the recommended, maintained fork that supports flat config natively. This avoids the need for compatibility wrappers like `@eslint/compat`.
- **Alternatives Considered**:
  - `eslint-plugin-import`: Rejected due to lack of native v9 support and potential stability issues.
  - `@eslint/compat`: Rejected to keep the configuration cleaner and dependency tree simpler.

## 2. TypeScript-ESLint Configuration

- **Decision**: Use `tseslint.config` (from `typescript-eslint`) or `defineConfig` (from `eslint/config`) for type safety.
- **Rationale**: While `tseslint.config` might be marked as deprecated in some contexts in favor of ESLint's own `defineConfig`, the `typescript-eslint` package exports a helper that is often still the most convenient for combining its own configs. _However_, recent searches suggest `tseslint.config` is deprecated in favor of `eslint/config`'s `defineConfig`.
- **Refinement**: We will use `typescript-eslint`'s recommended configuration arrays spread into the export, but wrapped in `tseslint.config(...)` if it simplifies the specific typescript-eslint typing, OR preferably just export the array directly if using `checkJs`/JSDoc for standard `eslint.config.js`. Given the findings, we will stick to the standard:
  ```js
  import tseslint from 'typescript-eslint';
  export default tseslint.config(...);
  ```
  _Correction_: The search result mentioned `tseslint.config` was deprecated, but `typescript-eslint` documentation typically recommends `tseslint.config()` as a helper to flatten config arrays easily. I will verify this during implementation, but `tseslint.config()` is the standard pattern in the v8/v9 transition docs for `typescript-eslint`. If deprecated, we will use the raw array export.

## 3. Turbo Task Orchestration

- **Decision**: Aggregate `check-style` depends on `check-eslint`, `check-biome`, `check-deps-check`.
- **Rationale**: Centralized control. Running `pnpm check-style` triggers all static analysis.
- **Caching**:
  - `check-eslint` inputs: `eslint.config.js`, `tsconfig.json`, `package.json`, source files.
  - `check-eslint` outputs: `[]` (Log only).

## 4. Staged Checks

- **Decision**: Sequential execution in `check-staged`.
- **Rationale**: Staged checks operate on a transient state (Git index). Parallel execution might race or produce garbled output.
- **Command**: `pnpm check-biome-staged && pnpm check-eslint-staged && pnpm check-filenames-staged`.
