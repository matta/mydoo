# Technology Stack: MyDoo

## Core Infrastructure

- **Language:** TypeScript
- **Monorepo Management:** pnpm workspaces, Turbo
- **Runtime:** Node.js (>=24)

## Frontend Application

- **Framework:** React (Vite)
- **State Management:**
  - **Local-First Sync:** Automerge (`@automerge/automerge-repo`)
  - **Application State:** Redux Toolkit
- **UI Toolkit:** Mantine (Current) / MUI (Future Consideration)
- **Network:** WebSocket (`automerge-repo-network-websocket`)
- **Persistence:** IndexedDB (`automerge-repo-storage-indexeddb`)

## Testing

- **Unit & Integration:** Vitest
- **End-to-End (E2E):** Playwright
- **Behavior-Driven Development (BDD):** `playwright-bdd`

## Code Quality & Tooling

- **Linter & Formatter:** Biome, ESLint, Prettier
- **Filename Convention:** ls-lint
- **Dependency Health:** knip
- **Package JSON Management:** syncpack
- **Git Hooks:** Husky
- **Commit Convention:** commitlint
