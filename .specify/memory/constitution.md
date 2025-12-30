<!--
Sync Impact Report:
- Version Change: 1.0.0 -> 1.1.0
- Modified Sections:
  - Development Workflow > Package Management: Renamed `pre-commit` to `check-staged`.
- Templates requiring updates:
  - None (validated via search)
-->
# mydoo Constitution

## Core Principles

### I. Fidelity First
Test environments must mirror production reality as closely as possible. Avoid simulated environments (like JSDOM) for core logic.
*   **Tier 1 (Logic):** Node.js (Native WASM performance).
*   **Tier 2 (Components):** Vitest Browser Mode (Real `IndexedDB`/`TextEncoder`).
*   **Tier 3 (Journeys):** Playwright (Full multi-tab sync, offline simulation).

### II. Executable Specifications
Tests are high-level narratives using the project's Ubiquitous Language.
*   **Ubiquitous Language:** ALWAYS use domain terms (`Inbox`, `Plan`, `Do`, `Balance`, `Context`). Reject implementation terms (`click`, `button`) in spec layers.
*   **Structure:** Use Gherkin-style `Given/When/Then` comments or steps to define intent clearly.

### III. Local-First Architecture
The application is client-centric, offline-capable, and relies on CRDTs (Automerge) for state.
*   **State:** Trust the local store (IndexedDB) as the source of truth.
*   **Sync:** Synchronization is a background process; the UI never blocks on network.

### IV. Strict Git Hygiene
Git operations must be explicit and safe.
*   **Clean Tree:** No work begins without a clean working directory.
*   **Explicit Commands:** `git commit` and `git push` REQUIRE explicit user command. No inference.
*   **Foreground Only:** Commits run synchronously to ensure hooks pass.

### V. Derived State Separation
Clear separation between Domain State and UI State.
*   **Selectors:** Use for domain concepts, shared data, or expensive calculations (Global).
*   **Hooks:** Use for tightly coupled local UI state or trivial transformations (Local).

## Testing Strategy

### 3-Tier Architecture
1.  **Pure Logic:** Test strictly in Node.js.
2.  **Components:** Test in Browser Mode (Goal). Mock `AutomergeRepo` networking if needed.
3.  **E2E:** Full journeys in Playwright.

## Development Workflow

### Package Management
*   Use `pnpm` for all operations.
*   Run `pnpm fix && pnpm check` before any commit request.

### Documentation
*   New code requires documentation comments explaining *why*, not just *what*.
*   Maintain `ROLLING_CONTEXT.md` for cross-session memory.

## Governance

### Supremacy
This Constitution supersedes all other practices. Conflicts must be resolved by amending this document.

### Amendments
*   **Major (X.0.0):** Principle removals or redefinitions.
*   **Minor (0.X.0):** New principles or guidance expansion.
*   **Patch (0.0.X):** Clarifications and fixes.

### Compliance
All Pull Requests and architectural decisions must be verified against these principles.

**Version**: 1.1.0 | **Ratified**: 2025-12-30 | **Last Amended**: 2025-12-30
