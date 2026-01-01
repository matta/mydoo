#!/bin/bash

# Get the list of staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACMR | grep -E "\.(js|jsx|ts|tsx|json|md|yaml|yml)$" || true)

if [ -z "$STAGED_FILES" ]; then
  echo "No staged files to check for formatting."
  exit 0
fi

echo "Checking formatted files..."
# Run prettier --check on staged files
# We use xargs to avoid "argument list too long" errors
echo "$STAGED_FILES" | xargs pnpm prettier --check
