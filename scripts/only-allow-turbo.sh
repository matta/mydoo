#!/bin/sh

TASK_NAME="$1"
shift

# Consume the separator if present
if [ "$1" = "--" ]; then
  shift
fi

# Guard: Check for Turbo environment
if [ -z "$TURBO_HASH" ]; then
  echo ""
  echo "⛔ Direct execution of '$TASK_NAME' is blocked."
  echo "   Run from the repo root to ensure prerequisites (lint/types) are met:"
  echo "   $ pnpm exec turbo run $TASK_NAME --filter=..."
  echo ""
  exit 1
fi

# Execute the actual command
exec "$@"
