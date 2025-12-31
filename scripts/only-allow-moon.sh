#!/bin/sh

TASK_NAME="$1"
shift

# Consume the separator if present
if [ "$1" = "--" ]; then
  shift
fi

# Guard: Check for Moon environment
if [ -z "$MOON_RUNNING" ]; then
  echo ""
  echo "⛔ Direct execution of '$TASK_NAME' is blocked."
  echo "   Run from the repo root to ensure prerequisites (lint/types) are met:"
  echo "   $ pnpm moon run :$TASK_NAME"
  echo ""
  exit 1
fi

# Execute the actual command
exec "$@"