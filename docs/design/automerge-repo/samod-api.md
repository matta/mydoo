> **Note:** This document describes the design of the external Rust `samod` crate (https://github.com/alexjg/samod) for reference purposes. It does not document the implementation within this repository.

# Client API Reference

## Overview

The `samod` library provides a robust framework for building collaborative, offline-first applications without requiring mandatory servers. It manages the lifecycle of `automerge` documents, handles persistent storage, and synchronizes data with peers over various network transports.

The core of the library is the `Repo` struct, which acts as a central registry for active documents ([`DocHandle`]) and manages network connections. The library is designed to be runtime-agnostic, supporting `tokio`, WASM (`wasm-bindgen`), and other async runtimes.

This API reference documents the public interface exposed by the `samod` crate, enabling clients to integrate document synchronization into their applications.

## Installation & Basic Usage

Add `samod` to your `Cargo.toml`:

```toml
[dependencies]
samod = { version = "0.1", features = ["tokio", "tungstenite"] } # Example features
```

### Basic Sync Example

```rust
use samod::{Repo, ConnDirection};

#[tokio::main]
async fn main() {
    // 1. Initialize the Repo on the tokio runtime
    let repo = Repo::build_tokio().load().await;

    // 2. Connect to a sync server or peer
    let socket = tokio::net::TcpStream::connect("sync.example.com:8080").await.unwrap();

    // Spawn the connection future (it must be driven to completion)
    tokio::spawn(repo.connect_tokio_io(socket, ConnDirection::Outgoing));

    // 3. Create a new document
    let initial_doc = automerge::Automerge::new();
    let doc_handle = repo.create(initial_doc).await.unwrap();

    println!("Document created with ID: {}", doc_handle.document_id());

    // 4. Modify the document
    doc_handle.with_document(|doc| {
        let mut tx = doc.transaction();
        tx.put(automerge::ROOT, "key", "value").unwrap();
        tx.commit();
    });
}
```

## Data Structures / Types

### `Repo`

The primary entry point for the library. It manages storage, networking, and document handles.

### `DocHandle`

A handle to a single, live `automerge` document. It provides valid concurrent access to the underlying data and exposes streams for observing changes.

### `RepoBuilder`

A builder interface for configuring and creating a `Repo`. Allows customization of storage backends, announce policies, and execution runtimes.

### `ConnDirection`

Enum indicating the direction of a network connection.

| Variant    | Description                                                               |
| :--------- | :------------------------------------------------------------------------ |
| `Incoming` | A connection accepted from a remote peer (e.g., server accepting client). |
| `Outgoing` | A connection initiated by the local peer (e.g., client dialing server).   |

### `ConnFinishedReason`

Enum describing why a network connection successfully terminated.

| Variant                  | Description                                       |
| :----------------------- | :------------------------------------------------ |
| `Shutdown`               | The local `Repo` was shut down.                   |
| `WeDisconnected`         | The local peer proactively closed the connection. |
| `TheyDisconnected`       | The remote peer closed the connection.            |
| `ErrorReceiving(String)` | An error occurred while receiving data.           |
| `ErrorSending(String)`   | An error occurred while sending data.             |

### `ConcurrencyConfig`

Enum configuring how the repository processes concurrent documents.

| Variant                         | Description                                                          |
| :------------------------------ | :------------------------------------------------------------------- |
| `AsyncRuntime`                  | Run document tasks on the provided async runtime (default).          |
| `Threadpool(rayon::ThreadPool)` | Run document tasks on a dedicated rayon thread pool (CPU-intensive). |

### `PeerId`

A unique identifier for a peer in the sync network. Re-exported from `samod_core`.

### `DocumentId`

A unique identifier for a document. Re-exported from `samod_core`.

### `AutomergeUrl`

A URL representation of a document ID, compatible with `automerge-repo` JS library. Re-exported from `samod_core`.

---

## API Reference

### Module: `samod`

#### `Repo::builder`

```rust
pub fn builder<R: RuntimeHandle>(runtime: R) -> RepoBuilder<InMemoryStorage, R, AlwaysAnnounce>
```

**Description**
Creates a generic `RepoBuilder` using a custom runtime handle.

**Parameters**

| Name      | Type               | Necessity | Description                                                         |
| :-------- | :----------------- | :-------- | :------------------------------------------------------------------ |
| `runtime` | `R: RuntimeHandle` | Required  | The runtime implementation to spawn tasks associated with the repo. |

**Return Value**
Returns a `RepoBuilder` configured with in-memory storage and an infinite announce policy.

---

#### `Repo::build_tokio`

```rust
#[cfg(feature = "tokio")]
pub fn build_tokio() -> RepoBuilder<InMemoryStorage, tokio::runtime::Handle, AlwaysAnnounce>
```

**Description**
Creates a `RepoBuilder` pre-configured for the current Tokio runtime. Requires the `tokio` feature.

**Panics**
Panics if called outside the context of a Tokio runtime.

**Return Value**
Returns a `RepoBuilder` for Tokio.

---

#### `Repo::build_wasm`

```rust
#[cfg(feature = "wasm")]
pub fn build_wasm() -> RepoBuilder<InMemoryStorage, WasmRuntime, AlwaysAnnounce>
```

**Description**
Creates a `RepoBuilder` pre-configured for WASM environments using `wasm-bindgen-futures`. Requires the `wasm` feature.

**Return Value**
Returns a `RepoBuilder` for WASM.

---

#### `Repo::load`

```rust
pub async fn load(self) -> Repo
```

**Description**
Consumes the `RepoBuilder` and instantiates the `Repo`. This starts the internal actor loops.

**Return Value**
Returns a fully initialized `Repo` instance.

---

#### `Repo::create`

```rust
pub async fn create(&self, initial_content: Automerge) -> Result<DocHandle, Stopped>
```

**Description**
Creates a new document with the provided initial content and persists it to storage.

**Parameters**

| Name              | Type        | Necessity | Description                        |
| :---------------- | :---------- | :-------- | :--------------------------------- |
| `initial_content` | `Automerge` | Required  | The initial state of the document. |

**Return Value**
Returns `Ok(DocHandle)` on success. Returns `Err(Stopped)` if the repo has been shut down.

---

#### `Repo::find`

```rust
pub fn find(&self, doc_id: DocumentId) -> impl Future<Output = Result<Option<DocHandle>, Stopped>>
```

**Description**
Locates a document by ID. It searches local storage first, then queries connected peers. The future resolves once the document is found locally or synced from a peer.

**Parameters**

| Name     | Type         | Necessity | Description                         |
| :------- | :----------- | :-------- | :---------------------------------- |
| `doc_id` | `DocumentId` | Required  | The ID of the document to retrieve. |

**Return Value**
Returns `Ok(Some(DocHandle))` if found. Returns `Ok(None)` if not found after querying all peers. Returns `Err(Stopped)` if the repo shuts down.

---

#### `Repo::connect`

```rust
pub fn connect<Str, Snk, ...>(
    &self,
    stream: Str,
    sink: Snk,
    direction: ConnDirection
) -> impl Future<Output = ConnFinishedReason>
```

**Description**
Connects a generic stream and sink to the repo's sync protocol. This is the low-level method powering other connect variants.

**Parameters**

| Name        | Type                                | Necessity | Description                                          |
| :---------- | :---------------------------------- | :-------- | :--------------------------------------------------- |
| `stream`    | `Stream<Item = Result<Vec<u8>, E>>` | Required  | Stream of incoming binary messages.                  |
| `sink`      | `Sink<Vec<u8>, Error = E>`          | Required  | Sink for outgoing binary messages.                   |
| `direction` | `ConnDirection`                     | Required  | The direction of the connection (Incoming/Outgoing). |

**Return Value**
Returns a future that drives the connection. Resolves to `ConnFinishedReason` when the connection ends.

**Edge Cases**
If the future is dropped, the connection is closed immediately.

---

#### `Repo::connect_tokio_io`

```rust
#[cfg(feature = "tokio")]
pub fn connect_tokio_io<Io>(
    &self,
    io: Io,
    direction: ConnDirection
) -> impl Future<Output = ConnFinishedReason>
```

**Description**
A convenience wrapper for Tokio `AsyncRead + AsyncWrite` streams (like TCP). Handles framing messages with a length delimiter.

**Parameters**

| Name        | Type                         | Necessity | Description               |
| :---------- | :--------------------------- | :-------- | :------------------------ |
| `io`        | `Io: AsyncRead + AsyncWrite` | Required  | The underlying IO stream. |
| `direction` | `ConnDirection`              | Required  | Incoming or Outgoing.     |

**Return Value**
Returns a connection driver future resolving to `ConnFinishedReason`.

---

#### `Repo::connect_wasm_websocket`

```rust
#[cfg(feature = "wasm")]
pub async fn connect_wasm_websocket(
    &self,
    url: &str,
    direction: ConnDirection
) -> ConnFinishedReason
```

**Description**
Connects to a WebSocket URL in a WASM environment using the browser's native API.

**Parameters**

| Name        | Type            | Necessity | Description                      |
| :---------- | :-------------- | :-------- | :------------------------------- |
| `url`       | `&str`          | Required  | WebSocket URL (ws:// or wss://). |
| `direction` | `ConnDirection` | Required  | Incoming or Outgoing.            |

**Return Value**
Returns `ConnFinishedReason` when the connection terminates.

**Panics**
Panics if called outside a browser environment (e.g., inside Node.js).

---

#### `Repo::when_connected`

```rust
pub async fn when_connected(&self, peer_id: PeerId) -> Result<(), Stopped>
```

**Description**
Waits until a connection is established with a specific peer. Useful for coordination in tests or initial sync.

**Parameters**

| Name      | Type     | Necessity | Description              |
| :-------- | :------- | :-------- | :----------------------- |
| `peer_id` | `PeerId` | Required  | The peer ID to wait for. |

**Return Value**
Returns `Ok(())` when connected. Returns `Err(Stopped)` if the repo shuts down.

---

#### `Repo::stop`

```rust
pub fn stop(&self) -> impl Future<Output = ()>
```

**Description**
Gracefully stops the `Repo`. Waits for pending storage tasks to complete before shutting down document actors.

**Return Value**
Returns a future that resolves when shutdown is complete.

---

### Struct: `DocHandle`

#### `DocHandle::document_id`

```rust
pub fn document_id(&self) -> &DocumentId
```

**Description**
Returns references to the document's unique ID.

**Return Value**
`&DocumentId`

---

#### `DocHandle::with_document`

```rust
pub fn with_document<F, R>(&self, f: F) -> R
where F: FnOnce(&mut Automerge) -> R
```

**Description**
Provides synchronous access to the underlying `Automerge` document. This locks the document actor, so it should be used for short operations.

**Parameters**

| Name | Type     | Necessity | Description                                                              |
| :--- | :------- | :-------- | :----------------------------------------------------------------------- |
| `f`  | `FnOnce` | Required  | A closure that receives a mutable reference to the `Automerge` document. |

**Return Value**
Returns the result `R` of the closure.

**Errors/Panics**
May block the current thread if the document is being processed. In async contexts, prefer running this in `spawn_blocking`.

---

#### `DocHandle::changes`

```rust
pub fn changes(&self) -> impl Stream<Item = DocumentChanged>
```

**Description**
Returns a stream that emits an event whenever the document is modified (locally or remotely).

**Return Value**
`impl Stream<Item = DocumentChanged>`

---

#### `DocHandle::broadcast`

```rust
pub fn broadcast(&self, message: Vec<u8>)
```

**Description**
Sends an ephemeral broadcast message to all peers viewing this document. These messages are not persisted history, just transient signals (e.g., cursor positions).

**Parameters**

| Name      | Type      | Necessity | Description                      |
| :-------- | :-------- | :-------- | :------------------------------- |
| `message` | `Vec<u8>` | Required  | The binary payload to broadcast. |

---

### Module: `samod::websocket` (WASM)

#### `WasmWebSocket::connect`

```rust
pub async fn connect(url: &str) -> Result<Self, NetworkError>
```

**Description**
Establishes a new WebSocket connection in a WASM environment.

**Parameters**

| Name  | Type   | Necessity | Description            |
| :---- | :----- | :-------- | :--------------------- |
| `url` | `&str` | Required  | The URL to connect to. |

**Return Value**
Returns `Ok(WasmWebSocket)` on success, or `Err(NetworkError)` on failure.

#### `WasmWebSocket::send`

```rust
pub fn send(&self, data: Vec<u8>) -> Result<(), NetworkError>
```

**Description**
Queues a binary message to be sent over the WebSocket.

**Parameters**

| Name   | Type      | Necessity | Description          |
| :----- | :-------- | :-------- | :------------------- |
| `data` | `Vec<u8>` | Required  | Binary data to send. |

**Return Value**
Returns `Ok(())` if queued successfully.
