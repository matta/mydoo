#!/bin/sh

# Returns 0 (true) if the current environment is an AI agent, 1 (false) otherwise.
if [ -n "$ANTIGRAVITY_AGENT" ] || [ -n "$GEMINI_CLI" ]; then
  exit 0
else
  exit 1
fi
