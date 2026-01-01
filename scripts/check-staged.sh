#!/bin/sh

set -e

# Dispatch based on environment
if ./scripts/is-agent.sh; then
  echo "🤖 Agent detected. Running full quality gates (Turbo)..."

  # If NOT running as a git hook (signaled by PRE_COMMIT_SECRET_REQUIRED),
  # forbid the secret (to train agents not to set it unnecessarily).
  if [ -z "$PRE_COMMIT_SECRET_REQUIRED" ] && [ -n "$AGENT_GIT_COMMIT_SECRET" ]; then
    echo "❌ Error: AGENT_GIT_COMMIT_SECRET should NOT be set when running check-staged outside of git."
    echo "   The secret is only required when committing via git."
    exit 1
  fi

  pnpm exec turbo run check-agent
else
  echo "👤 Human detected. Running staged quality gates (Turbo)..."
  pnpm exec turbo run check-human
fi
