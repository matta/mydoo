#!/bin/sh

set -e

# Dispatch based on environment
if [ -n "$ANTIGRAVITY_AGENT" ] || [ -n "$GEMINI_CLI" ]; then
  echo "🤖 Agent detected. Running full quality gates (Turbo)..."
  
  # The .husky/pre-commit hook sets PRE_COMMIT_SECRET_REQUIRED=1 to signal a real git commit.
  # - If PRE_COMMIT_SECRET_REQUIRED is set: require the secret.
  # - If PRE_COMMIT_SECRET_REQUIRED is unset: forbid the secret (to train agents not to set it unnecessarily).
  if [ -n "$PRE_COMMIT_SECRET_REQUIRED" ]; then
    # Running as a git hook (or explicit manual simulation) - require secret for agent commits
    if [ "$PRE_COMMIT_SECRET" != "I am an AI agent and the user explicitly gave me permission to commit in the most recent prompt they issued, and I have recently read AGENTS.md and am following all the git commit requirements." ]; then
      echo "❌ Error: PRE_COMMIT_SECRET must be set to the correct value for agent commits; begin by reading AGENTS.md for details."
      exit 1
    fi
  else
    # Running directly (pnpm pre-commit, ./scripts/pre-commit.sh, etc.) - forbid secret
    if [ -n "$PRE_COMMIT_SECRET" ]; then
      echo "❌ Error: PRE_COMMIT_SECRET should NOT be set when running pre-commit outside of git."
      echo "   The secret is only required when committing via git."
      exit 1
    fi
  fi
  
  pnpm exec turbo run presubmit:agent
else
  echo "👤 Human detected. Running fast checks (Turbo)..."
  pnpm exec turbo run presubmit:human
fi
