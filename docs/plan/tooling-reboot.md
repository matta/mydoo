# Comprehensive Simplification Plan: The "Ex-Googler" Reset

**Objective:** Reduce the repository to the absolute minimum configuration required for a strict, high-performance monorepo. This removes ~12 config files and standardizes on Rust-based tooling (Biome) for 90% of the work.

## Phase 1: The Great Purge (Delete Configurations)

Remove all the specialized tool configurations. We are effectively "resetting" the repo's tooling.

1.  **Delete Config Files:**
    Run these commands from the root:

    ```bash
    rm .prettierrc.json
    rm .prettierignore
    rm .lintstagedrc.js
    rm .syncpackrc
    rm commitlint.config.mjs
    rm apps/client/eslint.config.js
    rm apps/client/tsconfig.app.json
    rm apps/client/tsconfig.node.json
    rm apps/client/tsconfig.test.json
    rm packages/tasklens/eslint.config.mjs
    rm -rf .husky
    ```

2.  **Uninstall "The Goo":**
    Remove the JS-based formatters, linters, and checkers.

    ```bash
    pnpm remove -w -D \
      prettier \
      eslint-plugin-perfectionist \
      eslint-plugin-sonarjs \
      eslint-plugin-unicorn \
      eslint-plugin-check-file \
      eslint-plugin-eslint-comments \
      eslint-plugin-react-hooks \
      eslint-plugin-react-refresh \
      @commitlint/cli \
      lint-staged

    pnpm --filter @mydoo/tasklens remove -D prettier
    ```

3.  **Install Biome:**
    ```bash
    pnpm add -w -D @biomejs/biome
    ```

---

## Phase 2: Collapse TypeScript Complexity

Simplify `apps/client` to use a single `tsconfig.json` for everything (Source, Tests, and Vite Configs). This mimics a Go workspace (one config per module).

**Action:** Overwrite `apps/client/tsconfig.json` with this unified configuration:

```json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "baseUrl": ".",
    "outDir": "./dist",
    "target": "ESNext",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,

    /* Bundler mode */
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",

    /* Linting */
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,

    /* Mixed Environment Support (Node + DOM) */
    "types": ["node", "vite/client", "vitest/globals"],

    /* Monorepo Paths */
    "paths": {
      "@/*": ["./src/*"],
      "@mydoo/tasklens": ["../../packages/tasklens/src/index.ts"],
      "@mydoo/tasklens/*": ["../../packages/tasklens/src/*"]
    }
  },
  "include": ["src", "tests", "vite.config.ts", "vitest.config.ts"],
  "references": [{"path": "../../packages/tasklens"}]
}
```

_Note: We added `"types": ["node"]` so that `vite.config.ts` (which runs in Node) doesn't complain about `process.env`._

---

## Phase 3: Setup Biome (The "Cargo" Replacement)

Create the single source of truth for formatting, sorting, and basic linting.

**Action:** Create `biome.json` in the root:

```json
{
  "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
  "vcs": {
    "enabled": true,
    "client": "git",
    "useIgnoreFile": true
  },
  "files": {
    "ignoreUnknown": true,
    "ignore": [
      "dist",
      "dev-dist",
      "build",
      "coverage",
      ".turbo",
      ".vercel",
      "node_modules",
      "pnpm-lock.yaml",
      "**/*.d.ts"
    ]
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "lineWidth": 80
  },
  "organizeImports": {
    "enabled": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "complexity": {
        "noExcessiveCognitiveComplexity": {
          "level": "error",
          "options": {"maxAllowedComplexity": 40}
        }
      },
      "correctness": {
        "noUnusedImports": "error",
        "noUnusedVariables": "error"
      }
    }
  },
  "javascript": {
    "formatter": {
      "quoteStyle": "single",
      "trailingComma": "all",
      "arrowParentheses": "asNeeded",
      "bracketSpacing": false
    }
  },
  "overrides": [
    {
      "include": ["apps/client/**"],
      "linter": {
        "rules": {
          "style": {
            "noRestrictedImports": {
              "level": "error",
              "options": {
                "paths": {
                  "@mydoo/tasklens/persistence/*": [
                    "Deep imports forbidden. Use public API."
                  ]
                }
              }
            }
          }
        }
      }
    },
    {
      "include": ["packages/tasklens/src/domain/**"],
      "linter": {
        "rules": {
          "style": {
            "noRestrictedImports": {
              "level": "error",
              "options": {
                "paths": {
                  "react": ["Domain logic must be pure."],
                  "@automerge/*": ["Domain logic must be pure."]
                }
              }
            }
          }
        }
      }
    }
  ]
}
```

---

## Phase 4: The "Safety Net" (Hybrid ESLint)

Keep a tiny ESLint config specifically for the heavy TypeScript analysis that Biome cannot do yet.

**Action:** Overwrite `eslint.config.mjs` in the root:

```javascript
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      '**/dist/**',
      '**/coverage/**',
      '**/node_modules/**',
      '**/*.config.*',
      '**/.turbo/**',
    ],
  },
  {
    // STRICT TYPE CHECKING ONLY
    files: ['**/*.ts', '**/*.tsx'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    extends: [...tseslint.configs.strictTypeChecked],
    rules: {
      // Turn off everything Biome handles
      '@typescript-eslint/consistent-type-definitions': 'off',
      '@typescript-eslint/array-type': 'off',
      '@typescript-eslint/no-unused-vars': 'off', // Biome handles this

      // Turn off rules that conflict with "Simplicity"
      '@typescript-eslint/prefer-nullish-coalescing': 'off',

      // KEEP THE CRITICAL SAFETY RULES
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
    },
  },
  {
    // Allow JS config files to exist without type checking
    files: ['**/*.js', '**/*.mjs', '**/*.cjs'],
    extends: [tseslint.configs.disableTypeChecked],
  },
);
```

---

## Phase 5: Rebuild Scripts & Hooks

Update `package.json` to remove the old tools and point to the new ones.

**Action:** Update `scripts` in root `package.json`:

```json
"scripts": {
  "build": "turbo build",
  "dev": "turbo dev",
  "fix": "pnpm run fix:deps && biome check --write .",
  "fix:deps": "syncpack fix-mismatches && syncpack format",
  "format": "biome format .",
  "lint": "pnpm run lint:fast && pnpm run lint:strict",
  "lint:fast": "biome check .",
  "lint:strict": "eslint .",
  "lint:deps": "syncpack lint",
  "test": "turbo test",
  "prepare": "husky"
}
```

**Action:** Update `scripts` in `apps/client/package.json`:

```json
"scripts": {
  "build": "tsc -b && vite build",
  "dev": "vite",
  "lint": "biome check . && eslint .",
  "preview": "vite preview",
  "test": "vitest run",
  "test:e2e": "playwright test"
}
```

**Action:** Update `scripts` in `packages/tasklens/package.json`:

```json
"scripts": {
  "build": "tsc --emitDeclarationOnly",
  "generate-types": "json2ts ./specs/compliance/schemas/test_case.schema.json ./specs/compliance/schemas/test_case.d.ts && biome format --write ./specs/compliance/schemas/test_case.d.ts",
  "lint": "biome check . && eslint .",
  "test": "vitest run --coverage",
  "typecheck": "tsc --noEmit"
}
```

**Action:** Re-initialize Husky (Simple pre-commit):

```bash
pnpm exec husky init
```

**Action:** Write to `.husky/pre-commit` (overwriting default):

```bash
#!/bin/sh
# Only run the fast tool on commit.
# We trust CI to catch deep type errors (slow).
pnpm biome check --staged --no-errors-on-unmatched
```

**Action:** Update `.vscode/settings.json` to use Biome:

```json
{
  "editor.defaultFormatter": "biomejs.biome",
  "editor.formatOnSave": true,
  "[json]": {
    "editor.defaultFormatter": "biomejs.biome"
  }
}
```

> **Note:** Team members should install the [Biome VS Code extension](https://marketplace.visualstudio.com/items?itemName=biomejs.biome) and uninstall or disable Prettier.

**Action:** Update `.github/workflows/ci.yml` to use new commands:

```yaml
# Replace "Format Check" step with Biome:
- name: Biome Check
  run: pnpm lint:fast

# Keep these unchanged:
- name: Lint Dependencies
  run: pnpm lint:deps

- name: Lint (TypeScript Strict)
  run: pnpm lint:strict
```

> The old `pnpm format:check` and `pnpm lint` steps are replaced. `lint:fast` runs Biome (format + lint), `lint:strict` runs ESLint type-checked rules.

---

## Phase 6: Verification

Run all quality gates to confirm the migration succeeded:

```bash
pnpm install
pnpm fix
pnpm build
pnpm test
pnpm lint
syncpack list-mismatches
```

All commands should exit with code 0.

---

## Notes

- **`packages/tasklens/tsconfig.json`** remains unchanged. It already uses `tsconfig.base.json` and has no per-package ESLint config after Phase 1.
- **`syncpack`** is retained for dependency version consistency. Run `syncpack list-mismatches` periodically or in CI.
- **`apps/client/tsconfig.test.json`** is deleted. The unified `tsconfig.json` now includes all test files via `"include": ["src", "tests", ...]`. Vitest uses `vitest.config.ts` which inherits from this.

## Cleanup: Catalog Entries

After migration, remove unused entries from `pnpm-workspace.yaml` catalogs:

```yaml
# DELETE these lines:
'@commitlint/cli': ...
eslint-plugin-check-file: ...
eslint-plugin-eslint-comments: ...
eslint-plugin-perfectionist: ...
eslint-plugin-sonarjs: ...
eslint-plugin-unicorn: ...
lint-staged: ...
prettier: ...
```
