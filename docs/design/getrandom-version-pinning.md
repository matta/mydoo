# getrandom Version Pinning

## Status

**Proposed / Blocked**

## Context

Renovate attempted to update the workspace-level `getrandom` dependency from `v0.3.x` to `v0.4.0` ([PR #411](https://github.com/matta/mydoo/pull/411)). This update failed to generate a valid `Cargo.lock` (Artifact update problem) because of a strict version conflict with the project's core dependencies.

## Conflict Details

The `automerge` crate (version `0.7.3`), which is fundamental to the `mydoo` synchronization and storage layer, has a hard requirement on `getrandom = "^0.3"`.

In Rust's SemVer implementation for `0.x.y` versions:

- `0.3.x` and `0.4.0` are considered **incompatible breaking changes**.
- Cargo cannot substitute `0.4.0` to satisfy a `^0.3` requirement.

### Why not just force the update?

While we could manually update `Cargo.toml` to `getrandom = "0.4"`, this would result in **dependency duplication**:

1. The workspace and direct dependencies (like `uuid`) would move to `0.4.0`.
2. `automerge` would continue to pull in `0.3.x` as a transitive dependency.

For a low-level crate like `getrandom`, especially in a **WASM environment**, this is highly undesirable:

- **JS Interop Conflicts:** Both versions may attempt to initialize or import the same `wasm-bindgen` symbols (e.g., for `Crypto.getRandomValues`).
- **Binary Bloat:** Duplicating cryptographic or entropy-generation logic increases the WASM bundle size.
- **Inconsistency:** Different parts of the application would rely on different entropy sources or configurations.

## Resolution Strategy

We will stay on `getrandom 0.3.x` until the following upstream dependencies are updated to support `v0.4.0`:

1.  **automerge:** Track progress on the [automerge-rs repository](https://github.com/automerge/automerge).
2.  **rand ecosystem:** Ensure `rand` and `rand_core` (currently at `0.9.x`) have fully stabilized their integration with `getrandom v0.4.0`.

## Future Action

Once `automerge` releases a version supporting `getrandom 0.4`, we should:

1. Update `automerge` in the workspace.
2. Allow Renovate to re-run the `getrandom` update.
3. Verify that `Cargo.lock` no longer contains multiple versions of `getrandom` in the `0.3/0.4` range.
