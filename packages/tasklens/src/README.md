# TaskLens (TypeScript Core)

The core algorithm and data store implementation for the TaskLens prioritization engine, written in
TypeScript.

## Features

- **Dynamic Prioritization**: Implements the 7-Pass Tunnel Algorithm for
  calculating task priority.
- **Automerge Backing**: Uses `automerge` CRDT for local-first data storage and
  synchronization.
- **Strict Typing**: Fully typed with TypeScript matching the [`ALGORITHM.md`](../../../docs/design/algorithm.md)
  specification.
