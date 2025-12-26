#!/bin/sh

set -e

# Dispatch based on environment
if [ -n "$ANTIGRAVITY_AGENT" ] || [ -n "$GEMINI_CLI" ]; then
  echo "🤖 Agent detected. Running full quality gates (Turbo)..."
  pnpm exec turbo run "//#check:staged" test test:e2e typecheck
else
  echo "👤 Human detected. Running fast checks (Turbo)..."
  pnpm exec turbo run "//#check:staged" "//#lint:staged" typecheck
fi


