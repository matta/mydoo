# Tasklens UI Style Guide

This project follows a strict "upstream fidelity" policy for Dioxus components.

## 🚨 Critical Styling Rules

If you are:

1.  Modifying files in `crates/tasklens-ui/src/dioxus_components/`
2.  Using `Input { class: ... }` or `Textarea { class: ... }`
3.  Adding inline `style: "..."` to vendored components

**You MUST read and follow:**
`[Dioxus Upstream Styling Guidance](../../../docs/guidance/dioxus-upstream-styling.md)`

## Quick Summary

- **Do NOT** pass `class` to `Input` or `Textarea` (it breaks upstream styles).
- **Do NOT** modify vendored components unless absolutely necessary (Pristine Policy).
- **DO** use wrapper elements and CSS parent selectors (e.g. `.wrapper :global(.input)`).
- **DO** prefer app-owned adapters (e.g. `AppInput`) for common patterns.
