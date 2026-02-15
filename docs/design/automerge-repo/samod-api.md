> **Note:** This document describes the design/protocol of the external `automerge-repo` package (Node.js) for reference purposes. It does not document the implementation within this repository.

# Samod API Design

> **Status:** Draft / Reverse Engineering Notes
> **Target:** `automerge-repo` compatibility layer for Rust

This document sketches the API surface for `samod`, a Rust crate intended to provide `automerge-repo` compatible functionality.

## Core Types

### `DocumentId`

A URL representation of a document ID, compatible with `automerge-repo` JS library. Re-exported from `samod_core`.

- **Format:** `automerge:<base58>`
- **Conversion:** `FromStr`, `Display`

### `Repo`

The main entry point.

```rust
pub struct Repo {
    // ...
}

impl Repo {
    pub fn new(config: RepoConfig) -> Self { ... }
    pub fn create(&self) -> DocHandle { ... }
    pub fn find(&self, id: DocumentId) -> DocHandle { ... }
}
```

### `DocHandle`

A handle to a document, allowing sync and modification.

```rust
pub struct DocHandle {
    // ...
}

impl DocHandle {
    pub fn with_doc<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Automerge) -> R
    { ... }

    pub fn change<F>(&self, f: F)
    where
        F: FnOnce(&mut Automerge)
    { ... }
}
```

## Storage Adapter Trait

Compatible with `automerge-repo` storage semantics.

```rust
#[async_trait]
pub trait StorageAdapter: Send + Sync + 'static {
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>, Error>;
    async fn save(&self, key: &str, data: &[u8]) -> Result<(), Error>;
    async fn remove(&self, key: &str) -> Result<(), Error>;
}
```

## Network Adapter Trait

Compatible with `automerge-repo` network message format (CBOR).

```rust
#[async_trait]
pub trait NetworkAdapter: Send + Sync + 'static {
    async fn send(&self, peer: &PeerId, message: Message) -> Result<(), Error>;
    // ...
}
```

## Protocol Compliance

The implementation aims to be bit-compatible with the `automerge-repo` sync protocol v1.

- **Handshake:** `join` message with version negotiation.
- **Sync:** Standard Automerge sync state machine.
- **Gossip:** `peer-candidate` exchange.
