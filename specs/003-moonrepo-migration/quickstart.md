# Quickstart: Moonrepo Migration

**Feature Branch**: `003-moonrepo-migration`

## Prerequisites

- **Proto**: The toolchain manager used by Moon.
    ```bash
    # Install proto
    curl -fsSL https://proto.moonrepo.dev/install.sh | bash
    ```

## Bootstrap

1.  **Install Tools**:
    ```bash
    # Navigate to repo root
    cd mydoo
    
    # Install moon and managed node/pnpm
    proto use
    ```

2.  **Verify Environment**:
    ```bash
    moon --version
    node --version # Should match .moon/toolchain.yml
    pnpm --version # Should match .moon/toolchain.yml
    ```

## Common Commands

| Old Turbo Command | New Moon Command | Description |
| :--- | :--- | :--- |
| `pnpm build` | `moon run :build` | Build all projects (or specific one if in dir) |
| `pnpm test` | `moon run :test` | Test all projects |
| `pnpm check` | `moon run :check` | Run all linters and checks |
| `pnpm dev` | `moon run :dev` | Start development servers |
| `turbo run <task>` | `moon run :<task>` | Run a specific task |
| N/A | `moon ci` | Run the CI pipeline locally (simulated) |
| N/A | `moon project <name>` | View project details and tasks |

## Troubleshooting

-   **"Command not found: moon"**: Ensure `proto` shims are in your PATH. Run `proto use` to refresh.
-   **"Node version mismatch"**: Moon manages its own Node version. Use `moon run ...` or `proto run node -- ...` to ensure you're using the correct one.
