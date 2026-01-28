# Technology Stack: MyDoo

## Core Infrastructure

- **Language:** Rust (Core Logic & UI), TypeScript (Tooling & E2E Tests)
- **Monorepo Management:** pnpm workspaces, Turbo, Cargo Workspaces
- **Runtime:** Node.js (>=24)

## Frontend Application

- **Framework:** Dioxus (Rust)
- **State Management:**
  - **Local-First Sync:** Samod (Rust-based Automerge wrapper)
  - **Application State:** Signals (Dioxus native)
- **UI Toolkit:** Tailwind CSS, Dioxus Primitives
- **Network:** WebSocket (Samod sync)
- **Persistence:** IndexedDB (via Samod/Rexie in WASM)

## Testing

- **Unit & Integration:** Cargo Test (Rust), Vitest (Scripts)
- **End-to-End (E2E):** Playwright
- **Behavior-Driven Development (BDD):** `playwright-bdd`

## Code Quality & Tooling

- **Linter & Formatter:** Biome, ESLint, Prettier
- **Filename Convention:** Custom Script (`scripts/lint-filenames.ts`)
- **Dependency Health:** knip
- **Package JSON Management:** syncpack
- **Git Hooks:** Husky
- **Commit Convention:** commitlint