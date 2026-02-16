# AGENTS.md

This subtree contains all `tasklens-ui` code and assets, including vendored
Dioxus components and vendored theme files.

## Required Reads

- Read [`dioxus-vendored-components-design.md`](../../docs/design/dioxus-vendored-components-design.md) **before running vendoring updates or changing vendored ownership boundaries**.
- Read [`dioxus-vendored-customization.md`](../../docs/guidance/dioxus-vendored-customization.md) **before changing vendored component behavior, API usage patterns, or wrapper composition**.
- Read [`dioxus-vendored-styling.md`](../../docs/guidance/dioxus-vendored-styling.md) **before editing `assets/dx-components-theme.css` or adding visual customization for vendored components**.

## Local Policy

- Prefer app-owned adapters/wrappers over direct vendor edits.
- Keep vendored component files and vendored theme assets pristine by default.
- Treat styling policy as delegated to the styling guidance document.

## Vendored Code Map

- `crates/tasklens-ui/src/dioxus_components/**`: vendored component modules and per-component CSS.
- `crates/tasklens-ui/assets/dx-components-theme.css`: vendored theme stylesheet.
- `crates/tasklens-ui/dioxus-vendor-components.toml`: vendoring config (source project, revision, component set, module path).
- `crates/tasklens-ui/Cargo.toml` and `Cargo.lock`: `dioxus-primitives` source-project pin coordination.
