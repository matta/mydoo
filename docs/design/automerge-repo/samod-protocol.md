> **Note:** This document describes the design/protocol of the external `automerge-repo` package (Node.js) for reference purposes. It does not document the implementation within this repository.

# Samod Protocol Implementation Notes

This document tracks the implementation status of the `automerge-repo` protocol within `samod`.

## Message Type Implementation

| Message Type                 | Direction | Status | Notes                                   |
| :--------------------------- | :-------- | :----- | :-------------------------------------- |
| `join`                       | In/Out    | ✅     | Handshake supported.                    |
| `leave`                      | In/Out    | ⚠️     | Parsing supported, handling incomplete. |
| `peer-candidate`             | In/Out    | ❌     | Not yet implemented.                    |
| `sync`                       | In/Out    | ✅     | Full Automerge sync.                    |
| `request`                    | In/Out    | ✅     | Document fetching.                      |
| `unavailable`                | In/Out    | ✅     | NAK support.                            |
| `ephemeral`                  | In/Out    | ✅     | Cursor/presence support.                |
| `remote-subscription-change` | In/Out    | ⚠️     | Partial support.                        |
| `remote-heads-changed`       | In/Out    | ⚠️     | Partial support.                        |

## Serialization

- **CBOR**: Using `serde_cbor` (or equivalent) to match JS implementation.
- **Keys**: All keys in the protocol maps are strings.

## Transport

- **WebSocket**: `samod-transport-websocket` implements the `NetworkAdapter` trait for Axum/Tokio.
- **MessageChannel**: Not planned (browser-only).

## Divergences from JS Implementation

1.  **Strict Typing**: Rust implementation enforces stricter types on PeerIDs and DocumentIDs.
2.  **Concurrency**: `Repo` is `Send + Sync` and uses Tokio tasks for background processing, whereas JS uses the event loop.
