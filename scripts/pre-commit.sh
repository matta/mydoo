#!/bin/sh

set -e

# Dispatch based on environment
if [ -n "$ANTIGRAVITY_AGENT" ] || [ -n "$GEMINI_CLI" ]; then
  echo "🤖 Agent detected. Running full quality gates (Turbo)..."
  pnpm exec turbo run presubmit:agent
else
  echo "👤 Human detected. Running fast checks (Turbo)..."
  pnpm exec turbo run presubmit:human
fi


