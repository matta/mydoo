# Track Specification: Discovery-Key Based Sync

## 1. Overview
This track refactors the TaskLens sync protocol to use a **Discovery Key** pattern. We are removing the "Master Key" (user-level grouping) and "Room" concepts in favor of secure, per-document synchronization.

**The Core Pattern:**
1.  **Identity:** The client possesses a Document Secret (the `doc_id` or key).
2.  **Routing:** The client derives a public `discovery_key` (e.g., hash of the secret) to identify the document on the network.
3.  **Privacy:** The server *only* sees the `discovery_key` and opaque encrypted blobs. It cannot read the data or know the raw Document Secret.

## 2. Functional Requirements

### 2.1 Protocol Updates (`tasklens-sync-protocol`)
- **Field Renaming:** Rename `sync_id` to `discovery_key` in all messages (`Hello`, `SubmitChange`, `ChangeOccurred`).
- **Generic Payload:** Change `payload` from `EncryptedBlob` to `Vec<u8>`.
    - This allows the client to define its own encryption wrapper (e.g., including nonces) without baking it into the transport protocol.
    - The server treats payload as raw bytes.

### 2.2 Server Updates (`tasklens-sync-server`)
- **Database Schema:** Rename `sync_id` column to `discovery_key`.
- **Routing Logic:**
    - Bind incoming WebSocket connections to the `discovery_key` provided in the `Hello` handshake.
    - Route `SubmitChange` messages to all *other* clients connected with the same `discovery_key`.
- **Cleanup:** Remove any legacy "Master Key" derivation logic from the server (if any existed; mostly client-side, but ensure server nomenclature is clean).

## 3. Non-Functional Requirements
- **Security:** The server must remain a "Zero-Knowledge" relay. It must not require or store the decryption keys.
- **Scalability:** The architecture remains 1-Connection-Per-Document (for now).

## 4. Acceptance Criteria
- [ ] **Protocol Definition:** `tasklens-sync-protocol` uses `discovery_key` and `Vec<u8>`.
- [ ] **Server Schema:** Database table `updates` uses `discovery_key`.
- [ ] **Routing Verification:** Integration tests confirm that clients connected to `discovery_key_A` do NOT receive messages sent to `discovery_key_B`.
- [ ] **Payload Integrity:** Integration tests confirm that arbitrary byte arrays (simulating encrypted blobs) are stored and forwarded bit-for-bit intact.
