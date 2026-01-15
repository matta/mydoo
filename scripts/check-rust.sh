#!/bin/bash
# Rust validation script - run before committing Rust code
# Integrates with pnpm verify via root package.json

echo "=== Rust Validation ==="

echo "1. Checking Rust formatting..."
if ! cargo fmt --check; then
    echo "❌ Formatting failed. Fix with: cargo fmt"
    exit 1
fi

echo "2. Running Clippy..."
if ! cargo clippy --all-targets -- -D warnings; then
    echo "❌ Clippy failed. Fix with: cargo clippy --fix --allow-dirty"
    exit 1
fi

echo "3. Checking WASM build..."
if ! cargo build --target wasm32-unknown-unknown -p tasklens-store; then
    echo "❌ WASM build failed."
    exit 1
fi

echo "4. Building Dioxus UI..."
if ! dx build -p tasklens-ui; then
    echo "❌ Dioxus build failed."
    exit 1
fi

echo "5. Running Rust tests..."
if ! cargo test; then
    echo "❌ Tests failed."
    exit 1
fi

echo "=== Rust Validation Complete ==="
