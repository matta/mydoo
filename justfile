# justfile

set shell := ["bash", "-c"]

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------

ui_pkg := "crates/tasklens-ui"
docs_pkg := "docs"
scripts_pkg := "scripts"
core_pkg := "crates/tasklens-core"
store_pkg := "crates/tasklens-store"

# List available commands
_default:
    @just --list

# -----------------------------------------------------------------------------
# Build Commands
# -----------------------------------------------------------------------------

# Build all packages
build: build-core build-store build-ui

# Build tasklens-ui
build-ui:
    @echo "🏗️ Building tasklens-ui..."
    cd {{ui_pkg}} && dx build --platform web

# Build tasklens-core
build-core:
    @echo "🏗️ Building tasklens-core..."
    cargo build -p tasklens-core

# Build tasklens-store
build-store:
    @echo "🏗️ Building tasklens-store..."
    cargo build -p tasklens-store

# -----------------------------------------------------------------------------
# Style & Linting Commands
# -----------------------------------------------------------------------------

# Run all style checks
check-style: check-format-root check-biome-root check-eslint-root check-filenames-root check-turbo-root check-format check-eslint

# Check formatting for root files
check-format-root:
    @echo "🔍 Checking root formatting..."
    pnpm prettier --check "*.{json,md,yaml,yml,js,ts,tsx,jsx,css,html}"

# Check biome for root
check-biome-root:
    @echo "🔍 Checking root biome..."
    pnpm biome check .

# Check eslint for root
check-eslint-root:
    @echo "🔍 Checking root eslint..."
    pnpm eslint . --max-warnings 0 --ignore-pattern '*/**'

# Check filenames
check-filenames-root:
    @echo "🔍 Checking filenames..."
    pnpm tsx scripts/lint-filenames.ts

# Check turbo config
check-turbo-root:
    @echo "🔍 Checking turbo config..."
    pnpm tsx scripts/check-turbo-inputs.ts

# Check formatting for all packages
check-format:
    @echo "🔍 Checking package formatting..."
    @echo "  - {{ui_pkg}}" && cd {{ui_pkg}} && pnpm prettier --check .
    @echo "  - {{docs_pkg}}" && cd {{docs_pkg}} && pnpm prettier --check .
    @echo "  - {{scripts_pkg}}" && cd {{scripts_pkg}} && pnpm prettier --check .

# Check eslint for all packages
check-eslint:
    @echo "🔍 Checking package eslint..."
    @echo "  - {{scripts_pkg}}" && cd {{scripts_pkg}} && pnpm eslint .
    @echo "  - {{ui_pkg}}" && cd {{ui_pkg}} && pnpm eslint .

# -----------------------------------------------------------------------------
# Type Checking Commands
# -----------------------------------------------------------------------------

# Run all type checks
check-types: check-types-root check-types-scripts check-types-ui

# Check types for root
check-types-root:
    @echo "🔍 Checking root types..."
    pnpm tsc --noEmit

# Check types for scripts
check-types-scripts:
    @echo "🔍 Checking scripts types..."
    cd {{scripts_pkg}} && pnpm tsc --noEmit

# Check types for tasklens-ui
check-types-ui:
    @echo "🔍 Checking ui types..."
    # UI types are checked during build/dx build, but we can run tsc if config exists
    cd {{ui_pkg}} && pnpm tsc --noEmit

# -----------------------------------------------------------------------------
# Core Audit Commands
# -----------------------------------------------------------------------------

# Run full dead code detection suite (Lints + Dependencies)
audit: lint udeps
    @echo "✅ Audit complete! No dead code detected."

# Check compilation for WASM target
check-wasm:
    cargo check --target wasm32-unknown-unknown --all-targets

# Check for unused internal code and unreachable public API
# Relies on your Cargo.toml [workspace.lints] settings
lint:
    # @echo "🔍 Scanning for unused code (Internal & Public API)..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check for unused dependencies in Cargo.toml
# Note: Requires nightly toolchain and cargo-udeps installed
udeps:
    # @echo "📦 Scanning for unused dependencies..."
    cargo +nightly udeps --all-targets --all-features

# -----------------------------------------------------------------------------
# Setup & Maintenance
# -----------------------------------------------------------------------------

# Install required tools (cargo-udeps)
install-tools:
    @echo "🛠️ Installing cargo-udeps..."
    rustup toolchain install nightly
    cargo install cargo-udeps --locked

# Fix code automatically where possible (standard cargo fix)
fix:
    cargo fix --workspace --allow-dirty --allow-staged
