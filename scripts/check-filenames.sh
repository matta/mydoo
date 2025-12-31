#!/bin/sh

# Colors
RED='\033[0;31m'
NC='\033[0m' # No Color

STAGED_MODE=false
if [ "$1" = "--staged" ]; then
  STAGED_MODE=true
  echo "Checking staged filenames for kebab-case compliance..."
else
  echo "Checking filenames for kebab-case compliance..."
fi

# Function to get files to check
get_files() {
  if [ "$STAGED_MODE" = true ]; then
    git diff --name-only --cached --diff-filter=ACMR
  else
    find . -type f \( \
        -name "*.md" -o \
        -name "*.sh" -o \
        -name "*.yaml" -o \
        -name "*.yml" -o \
        -name "*.toml" -o \
        -name "*.config.js" -o \
        -name "*.config.ts" \
      \) \
      -not -path "./.git/*" \
      -not -path "*/.git/*" \
      -not -path "./node_modules/*" \
      -not -path "*/node_modules/*" \
      -not -path "./dist/*" \
      -not -path "*/dist/*" \
      -not -path "./dev-dist/*" \
      -not -path "*/dev-dist/*" \
      -not -path "./build/*" \
      -not -path "*/build/*" \
      -not -path "./coverage/*" \
      -not -path "*/coverage/*" \
      -not -path "./.moon/*" \
      -not -path "*/.moon/*" \
      -not -path "./.husky/*" | sed 's|^./||'
  fi
}

# The core logic:
# 1. Find all relevant files.
# 2. Filter for those that have Uppercase or Underscore in the filename itself.
# 3. Exclude (grep -v) specifically allowed patterns.
VIOLATIONS=$(get_files | grep -E "/?[^/]+[A-Z_][^/]*$" | \
  grep -v -E "/?AGENTS.md$" | \
  grep -v -E "/?README.md$" | \
  grep -v -E "/?CONTRIBUTING.md$" | \
  grep -v -E "/?TODO.md$" | \
  grep -v -E "/?GEMINI.md$" | \
  grep -v -E "/?ROLLING_CONTEXT.md$" | \
  grep -v -E "/?MASTER_PLAN.md$" | \
  grep -v -E "/?PHASE_[0-9]+_[A-Z_]+\.md$" | \
  grep -v -E "/?architecture_variances.md$" | \
  grep -v -E "/?test_case\.d\.ts$")

if [ -n "$VIOLATIONS" ]; then
  if [ "$STAGED_MODE" = true ]; then
    echo "${RED}❌ Error: The following staged files do not follow the kebab-case naming convention:${NC}"
  else
    echo "${RED}❌ Error: The following files do not follow the kebab-case naming convention:${NC}"
  fi
  echo "$VIOLATIONS"
  echo "${RED}Please rename them to kebab-case (e.g., my-file.ts).${NC}"
  exit 1
else
  echo "✅ All filenames are compliant."
  exit 0
fi
