# Dioxus Components Migration Checklist (Active)

## Tracking Policy

- Track this migration in this checklist and `dioxus-components-migration.md`.
- Do not create or update `bd` items for routine migration slices unless the user explicitly asks for `bd` tracking.
- When slice status changes, update this file directly in the same change.

## Working Agreement (Prevent Slice Drift)

- Every active slice must include both:
  - component adoption work (vendor/integrate/remove legacy), and
  - callsite class cleanup work (or explicit deferred file list in this checklist).
- The active chunk must point to the highest-impact remaining debt, not only the next component name.
- Keep migration inventory/status in `dioxus-components-migration.md` synced in the same change as checklist updates.
- Do not introduce new DaisyUI classes.

## Completed Work (Condensed)

- Foundations complete: vendor branch workflow, pinned registry config, `module_path = "src/dioxus_components"`, `app_components` split, and `assets/app.css` wiring.
- Phase 1 complete: DaisyUI plugin removed and DaisyUI class/theme debt cleared (exit gates 1-4).
- Completed slices include: Button, Input, Checkbox, Select, Dialog, Collapsible, Badge, Card, Progress, Textarea, Toggle/Switch, Label, Slider, and app-owned Date/DateTime wrappers with CSS modules.
- CSS modules feasibility gate is complete for app-owned components.

For full historical detail on removed checklist items, inspect git history:

```bash
git log -- docs/design/dioxus-components-migration.todo.md
git show <commit>:docs/design/dioxus-components-migration.todo.md
```

## Current Critical Path (Execute Top To Bottom)

- [ ] Migrate remaining legacy Dioxus component implementations/imports out of `crates/tasklens-ui/src/components` incrementally by vendoring into `crates/tasklens-ui/src/dioxus_components` and migrating callsites.
- [ ] Keep `crates/tasklens-ui/src/components` only as a temporary compatibility shim until all imports are migrated, then remove it.
- [ ] Verify adopted components still match upstream styling/API expectations:
  - [ ] Navbar: re-sync with upstream formatting and CSS.
  - [ ] Popover: re-sync with upstream formatting and CSS.
- [ ] Keep vendored `dioxus_components` on upstream `style.css` + `document::Link` patterns (no CSS-module conversion in vendored code).

## Two-Phase DaisyUI Then Tailwind Exit Gates

Phase 1 (DaisyUI removal): complete.

- [x] Gate 1: No DaisyUI component-skin tokens in app source.
- [x] Gate 2: No DaisyUI theme utility tokens in app source.
- [x] Gate 3: DaisyUI plugin usage removed from Tailwind configuration.
- [x] Gate 4: `just verify` passed after DaisyUI removal.

Phase 2 (Tailwind removal): remaining.

- [ ] Gate 5: Remove Tailwind inputs/outputs (`tailwind.css`, `assets/tailwind.css`, stylesheet link in `src/main.rs`).
- [ ] Gate 6: Restore pristine upstream `dx-components-theme.css`; keep app overrides in `assets/app.css`.
- [ ] Gate 7: Run `just verify` successfully after Tailwind removal.

## Audit Command

```bash
rg -n 'class:\s*(format!\(|format_args!\(|"[^"]*\b(btn|input|select|textarea|toggle|card|badge|progress|dropdown|menu|modal|loading|fieldset|join|bg-base-|text-base-|border-base-|text-primary)\b[^"]*")' crates/tasklens-ui/src --glob '*.rs'
```
