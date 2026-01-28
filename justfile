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
check-style: check-format-root check-biome-root check-eslint-root check-filenames-root check-format check-eslint

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
# Rust Commands
# -----------------------------------------------------------------------------

# Run all rust checks
check-rust: check-rust-fmt check-clippy check-wasm

# Check rust formatting
check-rust-fmt:
    @echo "🔍 Checking rust formatting..."
    cargo fmt --all -- --check

# Check clippy
check-clippy:
    @echo "🔍 Checking clippy..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check compilation for WASM target
check-wasm:
    @echo "🔍 Checking wasm compilation..."
    cargo check --target wasm32-unknown-unknown --all-targets

# -----------------------------------------------------------------------------
# Test Commands
# -----------------------------------------------------------------------------

# Run all unit tests
test: test-scripts test-rust

# Run unit tests for scripts
test-scripts:
    @echo "🧪 Running scripts tests..."
    cd {{scripts_pkg}} && pnpm test

# Run unit tests for rust crates
test-rust:
    @echo "🧪 Running rust tests..."
    cargo test --workspace

# Run all e2e tests
test-e2e: build-ui test-e2e-desktop test-e2e-mobile

# Run e2e tests for desktop
test-e2e-desktop:
    @echo "🧪 Running desktop e2e tests..."
    cd {{ui_pkg}} && pnpm exec playwright test --project=e2e-desktop

# Run e2e tests for mobile
test-e2e-mobile:
    @echo "🧪 Running mobile e2e tests..."
    cd {{ui_pkg}} && pnpm exec playwright test --project=e2e-mobile

# -----------------------------------------------------------------------------
# Orchestration Commands
# -----------------------------------------------------------------------------

# Full validation before human review
check-human: check-catalog-root check-deps-root check-syncpack-root check-style check-types

# Full CI-like validation
check-agent: check-human test test-e2e

# The "ultimate" verification command
verify: check-rust fix check-agent

# Development server
dev:
    @echo "🚀 Starting development server..."
    dx serve --platform web --package tasklens-ui

# -----------------------------------------------------------------------------
# Core Audit Commands
# -----------------------------------------------------------------------------

# Run full dead code detection suite (Lints + Dependencies)
audit: check-clippy check-catalog-root check-deps-root udeps
    @echo "✅ Audit complete! No dead code detected."

# Check for unused dependencies in Cargo.toml
# Note: Requires nightly toolchain and cargo-udeps installed
udeps:
    # @echo "📦 Scanning for unused dependencies..."
    cargo +nightly udeps --all-targets --all-features

# Check syncpack
check-syncpack-root:
    @echo "🔍 Checking syncpack..."
    pnpm syncpack list-mismatches

# Check catalog
check-catalog-root:
    @echo "🔍 Checking catalog..."
    pnpm tsx scripts/check-unused-catalog-entries.ts

# Check dependencies
check-deps-root:
    @echo "🔍 Checking dependencies..."
    pnpm knip

# -----------------------------------------------------------------------------
# Fix Commands
# -----------------------------------------------------------------------------

# Run all auto-fixes
fix: fix-style fix-syncpack-root fix-rust

# Run all style fixes
fix-style: fix-format-root fix-biome-root fix-eslint-root fix-format fix-biome fix-eslint

# Fix formatting for root files
fix-format-root:
    @echo "🛠️ Fixing root formatting..."
    pnpm prettier --write "*.{json,md,yaml,yml,js,ts,tsx,jsx,css,html}"

# Fix biome for root
fix-biome-root:
    @echo "🛠️ Fixing root biome..."
    pnpm biome check --write .

# Fix eslint for root
fix-eslint-root:
    @echo "🛠️ Fixing root eslint..."
    pnpm eslint . --fix --ignore-pattern '*/**'

# Fix formatting for all packages
fix-format:
    @echo "🛠️ Fixing package formatting..."
    @echo "  - {{ui_pkg}}" && cd {{ui_pkg}} && pnpm prettier --write .
    @echo "  - {{docs_pkg}}" && cd {{docs_pkg}} && pnpm prettier --write .
    @echo "  - {{scripts_pkg}}" && cd {{scripts_pkg}} && pnpm prettier --write .

# Fix biome for all packages
fix-biome:
    @echo "🛠️ Fixing package biome..."
    @echo "  - {{ui_pkg}}" && cd {{ui_pkg}} && pnpm biome check --write .

# Fix eslint for all packages
fix-eslint:
    @echo "🛠️ Fixing package eslint..."
    @echo "  - {{scripts_pkg}}" && cd {{scripts_pkg}} && pnpm eslint . --fix
    @echo "  - {{ui_pkg}}" && cd {{ui_pkg}} && pnpm eslint . --fix

# Fix syncpack
fix-syncpack-root:
    @echo "🛠️ Fixing syncpack..."
    pnpm syncpack fix-mismatches

# Fix rust (standard cargo fix)
fix-rust:
    @echo "🛠️ Fixing rust..."
    cargo fix --workspace --allow-dirty --allow-staged

# -----------------------------------------------------------------------------
# Setup & Maintenance
# -----------------------------------------------------------------------------

# Install required tools (cargo-udeps)
install-tools:
    @echo "🛠️ Installing cargo-udeps..."
    rustup toolchain install nightly
    cargo install cargo-udeps --locked
