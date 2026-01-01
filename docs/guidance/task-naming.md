# Task / Script Naming Specification for Turborepo / package.json

## 1. Syntax Constraints

- **Case:** Strictly `kebab-case`.
- **Characters:** `a-z`, `0-9`, and `-`.
- **Separators:** Segments are separated by a single hyphen.

## 2. Segment Hierarchy

`[action]-[subject]-[state]`

### I. Action (Required & Mutually Exclusive)

For static analysis and code hygiene, the `check` and `fix` actions distinguish read-only validation from mutation:

- **check:** READ-ONLY validation. Never modifies files.
- **fix:** MUTATION. Modifies source files to correct issues.

Additional action types:

- **test:** Runtime logic execution.
- **build:** Artifact generation.

Banned actions: `lint`, `format`, `verify`, `validate`, `check-*-fix`.

### II. Subject (Optional)

The domain being acted upon.

- **style:** Code patterns & quality (ESLint, Biome).
- **format:** Whitespace & layout (Prettier).
- **types:** Type definitions (TSC).
- **deps:** Dependencies (Knip).

Or, when specificity is needed, name by tool:

- **eslint** (for style)
- **biome** (for style)
- **tsc** (for types)
- **knip** (for deps)

Default to Domain names. When multiple tools cover one domain, name individual tasks by Tool and create an aggregate task using the Domain name.

### III. State (Optional)

The scope of files.

- **staged:** Git index.
- **changed:** Git diff.

## 3. The "One Way" Rules

1. **The Safety Rule:** For static analysis and code hygiene, distinguish safety explicitly. Use `fix` if the task modifies source files, and `check` if it is read-only.
2. **The Synonym Ban:** `lint` and `format` are banned as Actions. They are replaced by `check-style` / `fix-style` and `check-format` / `fix-format`.
3. **The Hybrid Ban:** `check-*-fix` is forbidden. A task cannot be both a check and a fix.

## 4. Canonical Examples

| Intent               | Old/Ambiguous Name | Unified Standard | Logic                                 |
| :------------------- | :----------------- | :--------------- | :------------------------------------ |
| **Verify Style**     | `lint`             | `check-style`    | Query: Is the style correct?          |
| **Correct Style**    | `lint-fix`         | `fix-style`      | Command: Make the style correct.      |
| **Verify Format**    | `format-check`     | `check-format`   | Query: Is the formatting correct?     |
| **Apply Format**     | `format`           | `fix-format`     | Command: Make the formatting correct. |
| **Type Check**       | `typecheck`        | `check-types`    | Query: Are types correct?             |
| **Dependency Check** | `lint-deps`        | `check-deps`     | Query: Are deps correct?              |
| **Pre-commit**       | `lint-staged`      | `check-staged`   | Command: Verify the staged files.     |
| **CI Validation**    | `ci`               | `check`          | Query: Is everything correct?         |

## 5. Exemptions

Common task names endemic to the npm/Node ecosystem are exempt when not explicitly banned above. Examples: `dev`, `start`, `serve`, `clean`, `prepare`, `prepublishOnly`.
