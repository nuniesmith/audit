#!/usr/bin/env bash
#
# Automated Repository Cache Scanner
#
# This script scans a repository and builds cache entries for all source files.
# It performs refactor analysis and documentation generation, storing results
# in the .rustassistant/cache/ directory.
#
# Usage:
#   ./scripts/cache_scan_repo.sh [REPO_PATH] [OPTIONS]
#
# Arguments:
#   REPO_PATH    Path to repository (default: current directory)
#
# Options:
#   --refactor   Only run refactor analysis (default: both)
#   --docs       Only run documentation generation (default: both)
#   --commit     Auto-commit cache files after scan
#   --push       Auto-push to remote after commit
#   --dry-run    Show what would be done without executing
#   --parallel N Run N analyses in parallel (default: 1)
#
# Example:
#   ./scripts/cache_scan_repo.sh ~/github/fks --commit
#   ./scripts/cache_scan_repo.sh . --refactor --parallel 3
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
REPO_PATH="${1:-.}"
RUN_REFACTOR=true
RUN_DOCS=true
AUTO_COMMIT=false
AUTO_PUSH=false
DRY_RUN=false
PARALLEL=1

# Parse arguments
shift 2>/dev/null || true
while [[ $# -gt 0 ]]; do
  case $1 in
    --refactor)
      RUN_REFACTOR=true
      RUN_DOCS=false
      shift
      ;;
    --docs)
      RUN_REFACTOR=false
      RUN_DOCS=true
      shift
      ;;
    --commit)
      AUTO_COMMIT=true
      shift
      ;;
    --push)
      AUTO_PUSH=true
      AUTO_COMMIT=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --parallel)
      PARALLEL="$2"
      shift 2
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

# Resolve absolute path
REPO_PATH=$(cd "$REPO_PATH" && pwd)

# Find rustassistant binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUSTASSISTANT="${PROJECT_ROOT}/target/debug/rustassistant"

if [[ ! -x "$RUSTASSISTANT" ]]; then
  echo -e "${RED}Error: rustassistant binary not found at $RUSTASSISTANT${NC}"
  echo -e "${YELLOW}Run: cargo build --bin rustassistant${NC}"
  exit 1
fi

# Check if .env exists
if [[ ! -f "$PROJECT_ROOT/.env" ]]; then
  echo -e "${RED}Error: .env file not found${NC}"
  echo -e "${YELLOW}Create $PROJECT_ROOT/.env with XAI_API_KEY${NC}"
  exit 1
fi

# Load environment
export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)

echo -e "${CYAN}================================================${NC}"
echo -e "${CYAN}  RustAssistant Cache Scanner${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""
echo -e "${BLUE}Repository:${NC}  $REPO_PATH"
echo -e "${BLUE}Refactor:${NC}    $RUN_REFACTOR"
echo -e "${BLUE}Docs:${NC}        $RUN_DOCS"
echo -e "${BLUE}Parallel:${NC}    $PARALLEL"
echo -e "${BLUE}Dry Run:${NC}     $DRY_RUN"
echo ""

# Initialize cache if needed
if [[ ! -d "$REPO_PATH/.rustassistant" ]]; then
  echo -e "${YELLOW}Initializing cache structure...${NC}"
  if [[ "$DRY_RUN" == "false" ]]; then
    "$RUSTASSISTANT" cache init --path "$REPO_PATH"
  else
    echo -e "${CYAN}[DRY RUN] Would run: rustassistant cache init${NC}"
  fi
  echo ""
fi

# Find all Rust source files (excluding target and .git)
echo -e "${YELLOW}Scanning for Rust files...${NC}"
mapfile -t RUST_FILES < <(find "$REPO_PATH" -name "*.rs" -type f \
  | grep -v "/target/" \
  | grep -v "/.git/" \
  | sort)

FILE_COUNT=${#RUST_FILES[@]}
echo -e "${GREEN}Found $FILE_COUNT Rust files${NC}"
echo ""

if [[ $FILE_COUNT -eq 0 ]]; then
  echo -e "${YELLOW}No Rust files found to analyze${NC}"
  exit 0
fi

# Statistics
ANALYZED=0
CACHED=0
ERRORS=0
SKIPPED=0

# Process files
cd "$REPO_PATH" || exit 1

process_file() {
  local file="$1"
  local rel_path="${file#$REPO_PATH/}"
  local result=0

  # Run refactor analysis
  if [[ "$RUN_REFACTOR" == "true" ]]; then
    if [[ "$DRY_RUN" == "false" ]]; then
      if "$RUSTASSISTANT" refactor analyze "$rel_path" > /dev/null 2>&1; then
        ((ANALYZED++)) || true
      else
        ((ERRORS++)) || true
        echo -e "${RED}  ✗ Error analyzing: $rel_path${NC}" >&2
        result=1
      fi
    else
      echo -e "${CYAN}[DRY RUN] Would analyze: $rel_path${NC}"
    fi
  fi

  # Run docs generation
  if [[ "$RUN_DOCS" == "true" ]]; then
    if [[ "$DRY_RUN" == "false" ]]; then
      if "$RUSTASSISTANT" docs module "$rel_path" > /dev/null 2>&1; then
        ((ANALYZED++)) || true
      else
        ((ERRORS++)) || true
        echo -e "${RED}  ✗ Error documenting: $rel_path${NC}" >&2
        result=1
      fi
    else
      echo -e "${CYAN}[DRY RUN] Would document: $rel_path${NC}"
    fi
  fi

  return $result
}

export -f process_file
export RUSTASSISTANT
export RUN_REFACTOR
export RUN_DOCS
export DRY_RUN
export ANALYZED
export ERRORS
export REPO_PATH
export RED GREEN YELLOW CYAN NC

echo -e "${YELLOW}Processing files...${NC}"

if [[ "$PARALLEL" -gt 1 ]]; then
  # Parallel processing
  echo -e "${BLUE}Running $PARALLEL analyses in parallel${NC}"
  printf '%s\n' "${RUST_FILES[@]}" | \
    xargs -P "$PARALLEL" -I {} bash -c 'process_file "$@"' _ {}
else
  # Sequential processing with progress
  CURRENT=0
  for file in "${RUST_FILES[@]}"; do
    ((CURRENT++)) || true
    rel_path="${file#$REPO_PATH/}"

    # Show progress
    echo -ne "${BLUE}[$CURRENT/$FILE_COUNT]${NC} $rel_path"

    # Check if already cached (for refactor)
    if [[ "$RUN_REFACTOR" == "true" ]] && [[ "$DRY_RUN" == "false" ]]; then
      if "$RUSTASSISTANT" refactor analyze "$rel_path" 2>&1 | grep -q "Using cached"; then
        echo -e " ${GREEN}[cached]${NC}"
        ((CACHED++)) || true
        ((SKIPPED++)) || true
        continue
      fi
    fi

    # Process the file
    if process_file "$file"; then
      echo -e " ${GREEN}✓${NC}"
    else
      echo -e " ${RED}✗${NC}"
    fi
  done
fi

echo ""
echo -e "${CYAN}================================================${NC}"
echo -e "${CYAN}  Scan Complete${NC}"
echo -e "${CYAN}================================================${NC}"

# Show cache status
if [[ "$DRY_RUN" == "false" ]]; then
  echo ""
  "$RUSTASSISTANT" cache status --path "$REPO_PATH"
fi

# Statistics
echo ""
echo -e "${BLUE}Statistics:${NC}"
echo -e "  Total files:     $FILE_COUNT"
echo -e "  Analyzed:        ${GREEN}$ANALYZED${NC}"
echo -e "  Cached (hit):    ${YELLOW}$CACHED${NC}"
echo -e "  Skipped:         $SKIPPED"
echo -e "  Errors:          ${RED}$ERRORS${NC}"

# Commit if requested
if [[ "$AUTO_COMMIT" == "true" ]] && [[ "$DRY_RUN" == "false" ]]; then
  echo ""
  echo -e "${YELLOW}Committing cache files...${NC}"

  cd "$REPO_PATH" || exit 1

  if git rev-parse --git-dir > /dev/null 2>&1; then
    git add .rustassistant/

    if git diff --staged --quiet; then
      echo -e "${YELLOW}No changes to commit${NC}"
    else
      COMMIT_MSG="chore: update rustassistant cache

- Scanned $FILE_COUNT Rust files
- Cached $ANALYZED new analyses
- $ERRORS errors encountered
- Generated by cache_scan_repo.sh"

      git commit -m "$COMMIT_MSG"
      echo -e "${GREEN}✓ Cache committed${NC}"

      if [[ "$AUTO_PUSH" == "true" ]]; then
        echo -e "${YELLOW}Pushing to remote...${NC}"
        git push
        echo -e "${GREEN}✓ Cache pushed${NC}"
      fi
    fi
  else
    echo -e "${RED}Error: Not a git repository${NC}"
    exit 1
  fi
fi

echo ""
if [[ $ERRORS -gt 0 ]]; then
  echo -e "${YELLOW}⚠  Completed with $ERRORS errors${NC}"
  exit 1
else
  echo -e "${GREEN}✓ All files processed successfully${NC}"
fi
