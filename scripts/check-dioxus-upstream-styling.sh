#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ALLOWLIST_FILE="$ROOT_DIR/scripts/dioxus-upstream-styling.allowlist"

TARGET_DIRS=(
  "$ROOT_DIR/crates/tasklens-ui/src/app_components"
  "$ROOT_DIR/crates/tasklens-ui/src/views"
)

BUILTIN_EXCLUDES=(
  "crates/tasklens-ui/src/app_components/form_controls.rs"
)

ALLOWLIST_GLOBS=()

if [[ ! -f "$ALLOWLIST_FILE" ]]; then
  echo "Missing allowlist file: $ALLOWLIST_FILE" >&2
  exit 1
fi

require_comment=0
line_num=0
while IFS= read -r raw_line || [[ -n "$raw_line" ]]; do
  ((line_num += 1))
  line="${raw_line#"${raw_line%%[![:space:]]*}"}"
  line="${line%"${line##*[![:space:]]}"}"

  if [[ -z "$line" ]]; then
    continue
  fi

  if [[ "${line:0:1}" == "#" ]]; then
    require_comment=1
    continue
  fi

  if [[ "$require_comment" -ne 1 ]]; then
    echo "Allowlist entry at line $line_num must be preceded by a rationale comment." >&2
    exit 1
  fi

  ALLOWLIST_GLOBS+=("$line")
  require_comment=0
done <"$ALLOWLIST_FILE"

is_excluded() {
  local relative_path="$1"
  local glob

  for glob in "${BUILTIN_EXCLUDES[@]}"; do
    if [[ "$relative_path" == $glob ]]; then
      return 0
    fi
  done

  if [[ ${#ALLOWLIST_GLOBS[@]} -gt 0 ]]; then
    for glob in "${ALLOWLIST_GLOBS[@]}"; do
      if [[ "$relative_path" == $glob ]]; then
        return 0
      fi
    done
  fi

  return 1
}

report_pattern_matches() {
  local kind="$1"
  local pattern="$2"
  local file="$3"

  local output
  if output="$(rg -nUP "$pattern" "$file")"; then
    echo "[dioxus-upstream-styling] $kind violation in $file"
    echo "$output"
    return 0
  fi

  return 1
}

CANDIDATE_FILES=()
while IFS= read -r candidate_file; do
  CANDIDATE_FILES+=("$candidate_file")
done < <(rg --files "${TARGET_DIRS[@]}" --glob '*.rs')

VIOLATIONS=0
for file in "${CANDIDATE_FILES[@]}"; do
  rel="${file#"$ROOT_DIR"/}"
  if is_excluded "$rel"; then
    continue
  fi

  if report_pattern_matches \
    "raw Input import" \
    '^\s*use\s+crate::dioxus_components::input::Input\s*;' \
    "$file"; then
    VIOLATIONS=1
  fi

  if report_pattern_matches \
    "raw Textarea import" \
    '^\s*use\s+crate::dioxus_components::textarea::Textarea\s*;' \
    "$file"; then
    VIOLATIONS=1
  fi

  if report_pattern_matches \
    "raw Input class usage" \
    '(?s)\bInput\s*\{[^}]*\bclass\s*:' \
    "$file"; then
    VIOLATIONS=1
  fi

  if report_pattern_matches \
    "raw Textarea class usage" \
    '(?s)\bTextarea\s*\{[^}]*\bclass\s*:' \
    "$file"; then
    VIOLATIONS=1
  fi

  if report_pattern_matches \
    "raw Input width workaround" \
    '(?s)\bInput\s*\{[^}]*\bstyle\s*:\s*"\s*width:\s*100%\s*;?\s*"' \
    "$file"; then
    VIOLATIONS=1
  fi
done

if [[ "$VIOLATIONS" -ne 0 ]]; then
  echo
  echo "[dioxus-upstream-styling] violations detected. See docs/guidance/dioxus-upstream-styling.md." >&2
  exit 1
fi

echo "[dioxus-upstream-styling] no violations found."
