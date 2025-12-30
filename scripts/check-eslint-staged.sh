#!/bin/sh

# Get staged JS/TS files
# Note: Unlike Biome's --staged, ESLint checks the entire file contents,
# not just the staged hunks. This is a limitation of ESLint.
STAGED_FILES=$(git diff --name-only --cached --diff-filter=ACMR | grep -E '\.(js|jsx|ts|tsx)$')

if [ -z "$STAGED_FILES" ]; then
  echo "No staged JS/TS files to lint."
  exit 0
fi

echo "Linting staged files..."
echo "$STAGED_FILES" | xargs pnpm exec eslint
