#!/usr/bin/env bash
#
# Simple Cache Scanner - Builds cache for specified files
#
# Usage:
#   ./scripts/simple_cache_scan.sh <repo_path> [file_pattern]
#
# Example:
#   ./scripts/simple_cache_scan.sh ~/github/fks "scripts/chaos-test/src/*.rs"
#

set -euo pipefail

REPO_PATH="${1:-.}"
SEARCH_PATH="${2:-.}"

# Find rustassistant binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUSTASSISTANT="${PROJECT_ROOT}/target/debug/rustassistant"

# Load environment
export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)

# Change to repo directory
cd "$REPO_PATH"

echo "🔍 Scanning repository: $REPO_PATH"
echo "📁 Search path: $SEARCH_PATH"
echo ""

# Initialize cache
if [[ ! -d ".rustassistant" ]]; then
  echo "Initializing cache..."
  "$RUSTASSISTANT" cache init --path .
  echo ""
fi

# Find files
mapfile -t FILES < <(find "$SEARCH_PATH" -name "*.rs" -type f | grep -v "/target/" | grep -v "/.git/" | sort)

echo "Found ${#FILES[@]} files"
echo ""

SUCCESS=0
CACHED=0
FAILED=0

# Process each file
for file in "${FILES[@]}"; do
  # Remove leading ./
  rel_path="${file#./}"

  echo -n "Analyzing: $rel_path ... "

  if output=$("$RUSTASSISTANT" refactor analyze "$rel_path" 2>&1); then
    if echo "$output" | grep -q "Using cached"; then
      echo "✓ [cached]"
      ((CACHED++))
    else
      echo "✓ [new]"
      ((SUCCESS++))
    fi
  else
    echo "✗ [error]"
    ((FAILED++))
  fi
done

echo ""
echo "================================================"
echo "Summary:"
echo "  New analyses: $SUCCESS"
echo "  Cache hits:   $CACHED"
echo "  Failed:       $FAILED"
echo "================================================"
echo ""

# Show cache status
"$RUSTASSISTANT" cache status --path .
