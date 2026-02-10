# Dioxus Components Vendor Tree

This directory is vendor-managed content produced by the Dioxus Components installer workflow.
Canonical migration context lives in `docs/design/dioxus-components-migration.md`.

## Status and ownership

- Path: `crates/tasklens-ui/src/dioxus_components`
- Source: `dx components add` from `https://github.com/DioxusLabs/components` at a pinned revision.
- Branch model: pristine snapshots are created on `vendor/dioxus-components-pristine` and merged into working branches.
- Update command: `cargo xtask update-dioxus-components`
- Edit policy: treat files here as upstream/vendor material. Keep local edits to the minimum strictly required by product needs.

In practice, manual edits in this tree should be rare because they increase future merge conflict risk when new vendor snapshots are merged.

## `dioxus-primitives` status

There is currently no vendored `dioxus_primitives/` source directory in this repository.

- Current model: `dioxus-primitives` is a pinned git dependency in `crates/tasklens-ui/Cargo.toml`.
- Pin consistency is validated against `Cargo.lock`.
- Intended behavior: update the dependency pin in lockstep with component vendoring.

## Vendored code map in `tasklens-ui`

### Pristine vendor-managed paths

- `crates/tasklens-ui/src/dioxus_components/**`
  - Installer-produced component modules and per-component CSS.
- `crates/tasklens-ui/assets/dx-components-theme.css`
  - Upstream theme asset used with Dioxus Components.
- `crates/tasklens-ui/dioxus-vendor-components.toml`
  - Source-of-truth `module_path`, registry `git`/`rev`, and component selection for vendoring.

### Upstream-derived but locally owned paths (not pristine snapshots)

- `crates/tasklens-ui/src/components/navbar/**` (adopted, minor formatting diffs)
- `crates/tasklens-ui/src/components/popover/**` (adopted, minor formatting diffs)
- `crates/tasklens-ui/src/components/slider/**` (adopted, minor formatting diffs)
- `crates/tasklens-ui/src/components/calendar/**` (wrapper/modified)
- `crates/tasklens-ui/src/components/collapsible/**` (wrapper/modified)
- `crates/tasklens-ui/src/components/dialog/**` (wrapper/modified)
- `crates/tasklens-ui/src/components/select/**` (wrapper/modified)
- `crates/tasklens-ui/src/components/button.rs` (diverged replacement)
- `crates/tasklens-ui/src/components/checkbox.rs` (diverged replacement)
- `crates/tasklens-ui/src/components/input.rs` (diverged replacement)
- `crates/tasklens-ui/src/components/date_picker/**` (diverged replacement)

These modules are part of the migration surface and may share upstream ancestry, but they are not currently treated as pristine vendor snapshots.

### Related upstream pin (dependency, not vendored source)

- `crates/tasklens-ui/Cargo.toml`
  - `dioxus-primitives` is pinned to the same upstream repository/revision model used for component vendoring.
