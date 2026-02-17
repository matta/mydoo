# getrandom Version Pinning

## Status

**Updated (Feb 16, 2026)** - Workspace upgraded to `getrandom v0.4.x`.

## Context

The workspace and core dependencies (including `uuid`) have been upgraded to `getrandom v0.4.1`. However, because `automerge v0.7.3` still requires `getrandom v0.3.x`, the repository currently maintains both versions.

## Conflict Details

The `automerge` crate (version `0.7.3`), which is fundamental to the `mydoo` synchronization and storage layer, has a hard requirement on `getrandom = "^0.3"`.

In Rust's SemVer implementation for `0.x.y` versions:

- `0.3.x` and `0.4.0` are considered **incompatible breaking changes**.
- Cargo cannot substitute `0.4.0` to satisfy a `^0.3` requirement.

### Managing Duplication

While we aim for a single version, multiple versions are currently necessary to allow the rest of the workspace to use `v0.4.x` while `automerge` remains on `v0.3.x`.

To ensure WASM builds work correctly with both versions, we:

1.  Directly depend on `getrandom v0.4.1` in the workspace with the `wasm_js` feature.
2.  Explicitly pull `getrandom v0.3.4` (aliased as `getrandom_03` in `Cargo.toml`) with the `wasm_js` feature in `tasklens-core` to ensure transitive 0.3.x dependencies (like `automerge`) have the required feature enabled for WASM support.

## Resolution Strategy

We will maintain this dual-version setup until `automerge` releases a version supporting `getrandom v0.4.0`.

1.  **automerge:** Track progress on the [automerge-rs repository](https://github.com/automerge/automerge). Once supported, update `automerge` and remove the `getrandom_03` workaround.

## Future Action

Once `automerge` releases a version supporting `getrandom 0.4`, we should:

1. Update `automerge` in the workspace.
2. Remove `getrandom_03` from `workspace.dependencies` and any crate-level dependencies.
3. Verify that `Cargo.lock` no longer contains multiple versions of `getrandom` in the `0.3/0.4` range.
