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
    cd {{ui_pkg}} && dx build --platform web

# Build tasklens-ui for e2e in release mode
build-ui-release:
    cd {{ui_pkg}} && dx build --platform web --release --debug-symbols=false

# Build tasklens-ui for e2e in debug mode
build-ui-debug:
    cd {{ui_pkg}} && dx build --platform web

# Build tasklens-core
build-core:
    cargo build -p tasklens-core

# Build tasklens-store
build-store:
    cargo build -p tasklens-store

# -----------------------------------------------------------------------------
# Style & Linting Commands
# -----------------------------------------------------------------------------

# Run all style checks
check-style: check-format check-xtask-all check-biome-root

# Check formatting (all files from root)
check-format:
    pnpm prettier --check .

# Check biome for root
check-biome-root:
    pnpm biome check .

# Run all xtask checks
check-xtask-all:
    cargo xtask check-all

# -----------------------------------------------------------------------------
# Type Checking Commands
# -----------------------------------------------------------------------------

# Run all type checks
check-types: check-types-root check-types-scripts check-types-ui

# Check types for root
check-types-root:
    pnpm tsc --noEmit

# Check types for scripts
check-types-scripts:
    cd {{scripts_pkg}} && pnpm tsc --noEmit

# Check types for tasklens-ui
check-types-ui:
    cd {{ui_pkg}} && pnpm tsc --noEmit

# -----------------------------------------------------------------------------
# Rust Commands
# -----------------------------------------------------------------------------

# Run all rust checks
check-rust: check-rust-fmt check-clippy check-wasm

# Check rust formatting
check-rust-fmt:
    cargo fmt --all -- --check

# Check clippy
check-clippy:
    chronic cargo clippy --workspace --all-targets -- -D warnings

# Check compilation for WASM target
check-wasm:
    chronic cargo check --workspace --target wasm32-unknown-unknown

# -----------------------------------------------------------------------------
# Test Commands
# -----------------------------------------------------------------------------

# Run all unit tests
test: test-scripts test-rust

# Run unit tests for scripts (pass through vitest args after `--`)
test-scripts *args:
    cd {{scripts_pkg}} && pnpm test {{args}}

# Run unit tests for rust crates
test-rust:
    cargo test --workspace

# Run all e2e tests
test-e2e *args: build-ui-release
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/release/web/public pnpm exec playwright test {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# Run e2e tests for desktop
test-e2e-desktop *args: build-ui-release
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/release/web/public pnpm exec playwright test --project=e2e-desktop {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# Run e2e tests for mobile
test-e2e-mobile *args: build-ui-release
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/release/web/public pnpm exec playwright test --project=e2e-mobile {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# Run all e2e tests in debug mode
test-e2e-debug *args: build-ui-debug
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/debug/web/public pnpm exec playwright test {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# Run e2e tests for desktop in debug mode
test-e2e-desktop-debug *args: build-ui-debug
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/debug/web/public pnpm exec playwright test --project=e2e-desktop {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# Run e2e tests for mobile in debug mode
test-e2e-mobile-debug *args: build-ui-debug
    cd {{ui_pkg}} && WEB_DIST_DIR=../../target/dx/tasklens-ui/debug/web/public pnpm exec playwright test --project=e2e-mobile {{args}}; \
    e=$?; \
    cargo xtask fix-junit test-results/junit.xml {{ui_pkg}}; \
    exit $e

# -----------------------------------------------------------------------------
# Orchestration Commands
# -----------------------------------------------------------------------------

# Full validation (static analysis)
check: check-deps-root check-syncpack-root check-style check-types check-rust

# The "ultimate" verification command
verify: fix check test test-e2e

# Development server
dev:
    dx serve --platform web --package tasklens-ui

# -----------------------------------------------------------------------------
# Core Audit Commands
# -----------------------------------------------------------------------------

# Run full dead code detection suite (Lints + Dependencies)
audit: check-clippy check-xtask-all check-deps-root udeps

# Check for unused dependencies in Cargo.toml
# Note: Requires nightly toolchain and cargo-udeps installed
udeps:
    cargo +nightly udeps --all-targets --all-features

# Check syncpack
check-syncpack-root:
    pnpm syncpack list-mismatches

# Check dependencies
check-deps-root:
    pnpm knip

# -----------------------------------------------------------------------------
# Fix Commands
# -----------------------------------------------------------------------------

# Run all auto-fixes
fix: fix-style fix-syncpack-root fix-rust

# Run all style fixes
fix-style: fix-format fix-biome-root

# Fix formatting (all files from root)
fix-format:
    pnpm prettier --write .

# Fix biome for root
fix-biome-root:
    pnpm biome check --write .

# Fix syncpack
fix-syncpack-root:
    pnpm syncpack fix-mismatches

# Fix rust (standard cargo fix)
fix-rust:
    cargo fmt
    cargo fix --workspace --allow-dirty --allow-staged

# -----------------------------------------------------------------------------
# Setup & Maintenance
# -----------------------------------------------------------------------------

# Install required tools (cargo-udeps)
install-tools:
    rustup toolchain install nightly
    cargo install cargo-udeps --locked
