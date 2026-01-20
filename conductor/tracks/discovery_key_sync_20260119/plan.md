# Implementation Plan - Discovery-Key Sync

## Phase 1: Protocol Refactoring (`tasklens-sync-protocol`) [checkpoint: 55d84ea]
- [x] Task: Update `ClientMessage` Enums 769cc28
    - [x] Rename `sync_id` to `discovery_key` in `Hello`.
    - [x] Rename `sync_id` to `discovery_key` in `SubmitChange`.
    - [x] Change `SubmitChange.payload` to `Vec<u8>`.
- [x] Task: Update `ServerMessage` Enums 769cc28
    - [x] Rename `sync_id` to `discovery_key` in `ChangeOccurred`.
    - [x] Change `ChangeOccurred.payload` to `Vec<u8>`.
- [x] Task: Conductor - User Manual Verification 'Protocol Refactoring' (Protocol in workflow.md)

## Phase 2: Server Implementation (`tasklens-sync-server`)
- [ ] Task: Update Database Layer (`db.rs`)
    - [ ] Rename `sync_id` column to `discovery_key` in `init_pool`.
    - [ ] Update `append_update` to take `discovery_key` and `Vec<u8>`.
    - [ ] Update `get_changes_since` to filter by `discovery_key`.
- [ ] Task: Update Request Handling (`main.rs`)
    - [ ] Update `ws_handler` to parse `discovery_key` from `Hello`.
    - [ ] Update connection loop to route based on `discovery_key`.
    - [ ] Ensure `payload` is passed through as opaque bytes.
- [ ] Task: Conductor - User Manual Verification 'Server Implementation' (Protocol in workflow.md)

## Phase 3: Verification
- [ ] Task: Update Integration Tests (`tests/integration_test.rs`)
    - [ ] Refactor `test_sync_flow` to use `discovery_key`.
    - [ ] Refactor `test_room_isolation` (rename variables to match new terminology).
    - [ ] Verify `payload` integrity with arbitrary bytes.
- [ ] Task: Conductor - User Manual Verification 'Verification' (Protocol in workflow.md)
