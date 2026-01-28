# Technology Stack: MyDoo

## Core Infrastructure

- **Language:** Rust (Core Logic & UI), TypeScript (Tooling & E2E Tests)
- **Monorepo Management:** pnpm workspaces, Just, Cargo Workspaces
- **Runtime:** Node.js (>=24)

## Frontend Application

- **Framework:** Dioxus (0.7.x)
- **Language:** Rust (WASM)
- **Styling:** Tailwind CSS

## Persistence & Sync

- **CRDT Library:** Automerge
- **Persistence:** IndexedDB (Browser)
- **Sync Protocol:** WebSockets (Custom)

## Quality Assurance

- **Unit & Integration:** Cargo Test (Rust), Vitest (Scripts)
- **End-to-End (E2E):** Playwright
- **End-to-End Style:** Code-First Gherkin (TypeScript)

## Code Quality & Tooling

- **Linter & Formatter:** Biome, ESLint, Prettier
- **Filename Convention:** Custom Script (`scripts/lint-filenames.ts`)
- **Dependency Health:** knip
- **Package JSON Management:** syncpack
- **Git Hooks:** Husky
- **Commit Convention:** commitlint
