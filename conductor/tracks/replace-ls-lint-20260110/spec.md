# Specification: Replace ls-lint with Custom TypeScript Script

## 1. Overview
Replace the binary dependency `@ls-lint/ls-lint` with a custom TypeScript script located in `scripts/`. This script will enforce filename conventions by consulting the existing `.ls-lint.yml` configuration but will strictly operate on files tracked by git.

## 2. Functional Requirements

### 2.1. Configuration Parsing
- The script **MUST** read and parse the existing `.ls-lint.yml` file at the project root.
- It **MUST** support the `ls` section to map extensions/globs to casing rules.
- It **MUST** support the `ignore` section to exclude specific file paths or patterns using standard glob matching.

### 2.2. File Discovery
- The script **MUST** use `git ls-files` to generate the list of files to check.
- It **MUST NOT** scan untracked files.

### 2.3. Validation Logic
- For each tracked file:
    1.  **Ignore Check:** Use `minimatch` to check if the file matches any pattern in the `ignore` section. If it matches, skip it.
    2.  **Rule Lookup:** Determine the applicable rule from the `ls` section.
    3.  **Validation:** Validate the filename against the required casing.
- **Casing Rules:** `kebab-case`, `snake_case`, `camelCase`, `PascalCase`, `regex:<pattern>`, `SCREAMING_SNAKE_CASE`.

### 2.4. Reporting
- Output error messages for violations.
- Exit code `1` for violations, `0` for success.

## 3. Technical Implementation
- **Script Location:** `scripts/lint-filenames.ts`
- **Dependencies:**
    - `js-yaml`: For parsing `.ls-lint.yml`.
    - `minimatch`: For glob matching against `ignore` patterns and `ls` keys.
    - `child_process`: For `git ls-files`.
- **Integration:**
    - Update `package.json`: Replace `ls-lint` script.
    - Remove `@ls-lint/ls-lint` package.

## 4. Acceptance Criteria
- [ ] `pnpm check-filenames-root` runs successfully.
- [ ] Ignored files in `.ls-lint.yml` are correctly skipped.
- [ ] Violations in tracked files are caught.
- [ ] `@ls-lint/ls-lint` is removed.
