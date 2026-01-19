# Cross-Browser Sync E2E Test Plan

## Goal

Implement and verify cross-browser synchronization for TaskLens using the
`tasklens-sync-server`. This ensures that multiple instances of the application
can collaborate on the same document in real-time.

## User Review Required

> [!IMPORTANT] **Sync Server Address Storage**: We need to decide where to store
> the user's preferred sync server address. For this MVP/test, `localStorage` is
> likely sufficient, separate from the Automerge document itself (since the
> server address is a local peer configuration, not a shared document property).

## Proposed Changes

### 1. Sync Client Integration (UI)

We need a WebSocket client in the UI that:

- Connects to the configured Sync Server.
- Performs the handshake (`Hello` message with `sync_id` i.e. DocId).
- Listens for remote changes and applies them via `store.import_changes()`.
- Observes local store changes and pushes them via `SubmitChange`.

#### Technical Approach

- Create a new hook `use_sync_client` in
  `crates/tasklens-ui/src/hooks/use_sync.rs` (or similar).
- This hook will:
  - Read the Sync Server URL from a Signal/Context.
  - Manage the WebSocket connection.
  - Buffer changes if offline.
  - Expose `sync_state` (Connected, Disconnected, Syncing).

### 2. User Settings for Sync

- **New Component**: `SyncSettings` modal or panel.
  - Input field for `Sync Server URL` (default: `ws://localhost:3000/sync`).
  - Button to `Connect`/`Disconnect`.
  - Persist choice to `localStorage` key `tasklens_sync_server_url`.
- **Status Indicator**:
  - Add a small icon/badge in the main layout (header/footer) showing connection
    status.
  - Clicking it opens the settings.

### 3. E2E Framework & Test Strategy

We will use **Playwright** with its ability to create multiple **Browser
Contexts**.

#### Test Scenario: `sync_basic.spec.ts`

1.  **Setup**:
    - Start `tasklens-sync-server` on a known port (e.g., 3001) as a background
      process in the test.
2.  **Context A (Alice)**:
    - Open TaskLens.
    - Create a new Document.
    - Configure Sync Server to `ws://localhost:3001/sync`.
    - Verify Status turns 🟢.
    - Create Task "Buy Milk".
3.  **Context B (Bob)**:
    - Open TaskLens (same URL with Doc ID).
    - Configure Sync Server.
    - Verify Status turns 🟢.
    - **Assert**: Task "Buy Milk" appears automatically.
4.  **Bidirectional**:
    - Bob checks "Buy Milk" as Done.
    - **Assert**: Alice sees "Buy Milk" as Done.

#### Technical Detail: Browser Context Isolation

We leverage Playwright's `browser.newContext()` to strictly isolate the two
sessions ("Alice" and "Bob"). Each context has its own:

- **LocalStorage**: This ensures they can store different user settings (though
  in this test they might use the same sync server URL, the _storage_ of that
  setting is separate).
- **IndexedDB**: The Automerge document is persisted here. Separation ensures
  that `Context A` cannot read `Context B`'s database directly; it _must_
  receive data via the Sync Server.
- **Session/Cookies**: Completely independent auth/session state.

#### Doc Initialization

- `doc_id` is the `sync_id`.
- Server replays history on connection. `AppStore::import_changes` handles
  merge.

## Verification Plan

### Automated Tests

- **New Playwright Spec**: `tests/e2e/specs/sync.spec.ts`
  - Implements the multi-context flow described above.
  - Uses a helper to spawn/kill the rust sync server binary.

### Manual Verification

- Run `cargo run -p tasklens-sync-server`.
- Open two browser tabs to `http://localhost:5180`.
- Connect both to `ws://localhost:3000/sync`.
- Observe changes propagating.
