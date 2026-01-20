#!/bin/bash

# CAUTION: This script aggressively cleans the repository of ignored files.
# It is useful for debugging CI issues locally by ensuring a pristine state.
# It purposefully EXCLUDES (preserves) files that are expensive to re-download or critical for local dev.

echo "Running git clean -fdX..."
echo "Excluding: ROLLING_CONTEXT.md, .proto (toolchain), .env, *.local"

# -f: force
# -d: recurse into untracked directories
# -X: remove only files ignored by Git
# -e: exclude pattern (using '!' to un-ignore files so -X leaves them alone)

git clean -fdX \
  -e '!ROLLING_CONTEXT.md' \
  -e '!.proto' \
  -e '!.env' \
  -e '!*.local' \
  -e '!.vscode' \
  -e '!.gemini' \
  -e '!.specify'

echo "Clean complete."
