# Samod Migration Plan & Status

## Goal
Port `tasklens-ui` and `tasklens-store` from a custom WebSocket sync implementation to `samod` (a Rust-native port of `automerge-repo`). This ensures cross-platform compatibility (WASM + Native) and robust synchronization.

## Current Status (2026-01-20)
*   **`tasklens-store`**: **COMPLETED & VERIFIED**.
    *   `AppStore` structure refactored to use `samod::Repo` and `samod::DocHandle`.
    *   Platform-specific guards (`#[cfg(target_arch = "wasm32")]`) removed from core logic, enabling native tests.
    *   Legacy `network.rs` module removed.
    *   Compiles cleanly. `cargo test -p tasklens-store` passes (with new static tests for `handle_create_task`).
    *   Resolved tricky compilation issues: `Transactable` trait ambiguity (used `am_*` helpers), `ExId` vs `ObjId`, and `DocumentId` type conversion.
*   **`tasklens-ui`**: **IMPLEMENTED, PENDING VERIFICATION**.
    *   `hooks/use_sync.rs` refactored to use `samod::Repo::connect_websocket`.
    *   `samod_storage.rs` fixed to correctly implement `Storage` trait for `Rexie` (using public `StorageKey` API).
    *   Basic `samod` dependency added.
    *   **Next Action**: Run `cargo build -p tasklens-ui --target wasm32-unknown-unknown` to verify these fixes.

## Implementation Details & Decisions

### Repository Structure
*   **`crates/tasklens-store/src/store.rs`**: The core `AppStore` now wraps `Option<samod::Repo>` and `Option<samod::DocHandle>`. It allows initialization `with_repo` for testing.
*   **`crates/tasklens-store/src/doc_id.rs`**: Implements `From<samod::DocumentId> for DocumentId` and vice-versa (via byte slice), bridging generic `tasklens` types with `samod`.
*   **`crates/tasklens-store/src/sync.rs`**: New module defining `SyncStatus`.
*   **`crates/tasklens-store/src/samod_storage.rs`**: Implements `samod::storage::LocalStorage` using `Rexie` (IndexedDB).

### Remaining Work Checklist

- [x] **Phase 1: Dependencies & Core Structs**
    - [x] Add `samod` dependency.
    - [x] Create `WasmRuntime` and `SamodStorage` adapters.
    - [x] Refactor `AppStore` to hold `samod::Repo`.

- [x] **Phase 2: Store Logic & Compilation**
    - [x] Refactor `AppStore` methods (`dispatch`, `create_new`, etc.) to use `DocHandle`.
    - [x] Fix `DocumentId` type mismatches.
    - [x] Solve `Transactable` trait ambiguity in `store.rs`.
    - [x] Verify `tasklens-store` compiles and tests pass.
    - [x] Verify `tasklens-store` test coverage is *identical* to what it was, by comparing the git diff and restoring missing tests.
    - [x] Verify `tasklens-store` restored tests pass.

- [x] **Phase 3: UI Integration (Completed)**
    - [x] Update `main.rs` to initialize `samod::Repo`.
    - [x] Refactor `use_sync` hook.
    - [x] **Verify `tasklens-ui` compilation** (WASM target).

- [ ] **Phase 4: E2E Testing & Verification**
    - [ ] Run `sync.spec.ts` (E2E tests).
        - *Note*: Sync server is now a Node.js script (`scripts/sync-server.mjs`) compatible with `automerge-repo`.
    - [ ] Verify persistence works in browser (IndexedDB via `samod_storage`).
    - [ ] Verify cross-tab/cross-browser sync.

- [ ] **Phase 5: Cleanup**
    - [ ] Remove any remaining dead code from `tasklens-store` (legacy files).
    - [ ] Standardize logging/tracing.

## How to Resume
1.  **Verify UI Build**: Run `cargo build -p tasklens-ui --target wasm32-unknown-unknown`. Isolate and fix any remaining errors (likely around `samod` interaction or missing features in UI crate).
2.  **Run E2E Tests**: Use `npx playwright test tests/e2e/sync.spec.ts`. This uses the new Node.js sync server.
3.  **Manual Check**: Open the app in browser (`trunk serve`), create tasks, verify they persist and sync.
