#!/bin/sh

# Colors
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Checking filenames for kebab-case compliance..."

# Create a temporary file for the grandfathered list
GRANDFATHERED_FILE=$(mktemp "${TMPDIR:-/tmp}/grandfathered_files.XXXXXX") || exit 1
trap 'rm -f "$GRANDFATHERED_FILE"' EXIT

# Files to ignore (Grandfathered list)
# We use a grep pattern file for this to handle the list cleanly
cat <<EOF > "$GRANDFATHERED_FILE"
./packages/tasklens/specs/compliance/fixtures/zero_feedback.yaml
./packages/tasklens/specs/compliance/fixtures/tree_order.yaml
./packages/tasklens/specs/compliance/fixtures/boost_lead_time.yaml
./packages/tasklens/specs/compliance/fixtures/visibility_place_filtering.yaml
./packages/tasklens/specs/compliance/fixtures/lead_time_edge_cases.yaml
./packages/tasklens/specs/compliance/fixtures/sequential_flow.yaml
./packages/tasklens/specs/compliance/fixtures/lead_time.yaml
./packages/tasklens/specs/compliance/fixtures/tree_order_id_conflict.yaml
./packages/tasklens/specs/compliance/fixtures/min_threshold.yaml
./packages/tasklens/specs/compliance/fixtures/complex_mutation.yaml
./packages/tasklens/specs/compliance/fixtures/boost_importance.yaml
EOF

# Find files that violate the convention
#
# Exclusions Strategy:
# 1. System/Generated: .git, node_modules, build artifacts (dist, coverage) - Not source code.
# 2. Source Code: Delegated to Biome.
#    Biome is configured to enforce `kebab-case` for ts, js, json, etc.
#    See `biome.json` > linter > rules > style > useFilenamingConvention.
VIOLATIONS=$(find . -type f \( \
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
  -not -path "./.turbo/*" \
  -not -path "*/.turbo/*" \
  -not -path "./.husky/*" \
  | grep -E "/[^/]*[A-Z_][^/]*$" \
  | grep -v -E "/[a-zA-Z0-9_.]+\.md$" \
  | grep -v -E "/[a-zA-Z0-9_.]+\.sh$" \
  | grep -vFf "$GRANDFATHERED_FILE")

if [ -n "$VIOLATIONS" ]; then
  echo "${RED}❌ Error: The following files do not follow the kebab-case naming convention:${NC}"
  echo "$VIOLATIONS"
  echo "${RED}Please rename them to kebab-case (e.g., my-file.ts).${NC}"
  exit 1
else
  echo "✅ All filenames are compliant."
  exit 0
fi
