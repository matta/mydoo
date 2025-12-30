#!/bin/sh

# Prevent ROLLING_CONTEXT.md from accidental commitment
if git diff --cached --name-only | grep -q "^ROLLING_CONTEXT.md$"; then
  echo "❌ ERROR: ROLLING_CONTEXT.md is staged for commit."
  echo "This file is for local working memory only and must not be committed."
  echo "Run 'git restore --staged ROLLING_CONTEXT.md' to unstage it."
  exit 1
fi
