# Research: Cross-Platform BDD Execution

**Feature**: `002-mobile-desktop-bdd`
**Date**: 2025-12-30

## Decisions

### 1. Configuration Strategy

**Decision**: Use Playwright `projects` to define "Desktop" and "Mobile" configurations within a single `playwright.config.ts`.
**Rationale**:

- **Native Support**: Playwright was designed to handle multi-environment testing via the `projects` array.
- **Code Reuse**: Allows reusing the `webServer`, `reporter`, and global `use` settings.
- **Maintenance**: Single file to manage.

### 2. Mobile Emulation Profile

**Decision**: Use `devices['Pixel 7']`.
**Rationale**:

- **Standard**: Represents a modern Android viewport (412x915) with high density.
- **Lightweight**: Uses Chrome's device emulation (User Agent + Viewport + Touch Events) rather than a heavy Android Emulator.
- **Fidelity**: Sufficient for web-based responsive design and touch interaction testing (Tier 3 fidelity).

### 3. CI Execution

**Decision**: Sequential execution (Desktop, then Mobile).
**Rationale**:

- **Simplicity**: Easier to read logs and identify which environment failed.
- **Resource Management**: Avoids potential resource contention on smaller CI runners compared to running 2x workers.
- **Trade-off**: Builds will take longer (~2x), but this was accepted in the specification.

### 4. BDD Integration

**Decision**: Share the `playwright-bdd` generated test directory across both projects.
**Rationale**:

- `defineBddConfig` generates standard Playwright test files.
- These files can be executed by multiple projects with different `use` parameters (viewport, user agent) without regeneration.

## Alternatives Considered

- **Separate Config Files**: Rejected. Harder to maintain shared settings.
- **Real Android Emulator**: Rejected. Too slow and resource-intensive for every PR; fidelity gain (native browser engine quirks) is marginal for a PWA.
- **Skipping Tests**: Rejected by Specification. All tests must pass on all platforms (Responsive Design).
