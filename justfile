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
