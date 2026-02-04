#!/usr/bin/env python3
"""
Batch Cache Scanner for RustAssistant

This script efficiently scans multiple Rust files and builds cache entries.
It handles file processing, progress tracking, and error reporting.

Usage:
    python3 scripts/batch_cache.py <repo_path> [--limit N] [--commit]

Examples:
    python3 scripts/batch_cache.py ~/github/fks
    python3 scripts/batch_cache.py ~/github/fks --limit 20
    python3 scripts/batch_cache.py ~/github/fks --commit
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple


def find_rust_files(repo_path: Path, exclude_patterns: List[str] = None) -> List[Path]:
    """Find all Rust source files in repository."""
    if exclude_patterns is None:
        exclude_patterns = ["/target/", "/.git/", "/examples/", "/benches/"]

    rust_files = []
    for rust_file in repo_path.rglob("*.rs"):
        # Check if any exclude pattern is in the path
        if any(pattern in str(rust_file) for pattern in exclude_patterns):
            continue
        rust_files.append(rust_file)

    return sorted(rust_files)


def get_rustassistant_path() -> Path:
    """Find the rustassistant binary."""
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    binary = project_root / "target" / "debug" / "rustassistant"

    if not binary.exists():
        print(f"❌ Error: rustassistant binary not found at {binary}")
        print("   Run: cargo build --bin rustassistant")
        sys.exit(1)

    return binary


def load_env(project_root: Path) -> dict:
    """Load environment variables from .env file."""
    env_file = project_root / ".env"
    if not env_file.exists():
        print(f"❌ Error: .env file not found at {env_file}")
        sys.exit(1)

    env = os.environ.copy()
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                if "=" in line:
                    key, value = line.split("=", 1)
                    env[key.strip()] = value.strip()

    # Set DATABASE_URL if not already set
    if "DATABASE_URL" not in env:
        # Use the default database path from rustassistant
        env["DATABASE_URL"] = "sqlite:data/rustassistant.db"

    return env


def analyze_file(
    rustassistant: Path, repo_path: Path, file_path: Path, env: dict
) -> Tuple[bool, bool, str]:
    """
    Analyze a single file with rustassistant.

    Returns:
        (success, cached, message)
    """
    try:
        rel_path = file_path.relative_to(repo_path)

        # Run analysis
        result = subprocess.run(
            [str(rustassistant), "refactor", "analyze", str(rel_path)],
            cwd=str(repo_path),
            env=env,
            capture_output=True,
            text=True,
            timeout=120,  # 2 minute timeout per file
        )

        output = result.stdout + result.stderr

        if result.returncode != 0:
            error_msg = output.split("\n")[0] if output else "Unknown error"
            return False, False, error_msg

        # Check if cached or new analysis
        if "Using cached" in output:
            return True, True, "cached"
        elif "Analysis cached" in output:
            return True, False, "new"
        else:
            return True, False, "analyzed"

    except subprocess.TimeoutExpired:
        return False, False, "timeout"
    except Exception as e:
        return False, False, str(e)


def print_progress(
    current: int, total: int, file_path: Path, status: str, cached: bool = False
):
    """Print progress with color coding."""
    percentage = (current / total) * 100

    # Color codes
    GREEN = "\033[0;32m"
    YELLOW = "\033[1;33m"
    RED = "\033[0;31m"
    BLUE = "\033[0;34m"
    NC = "\033[0m"  # No Color

    if cached:
        color = YELLOW
        symbol = "📦"
    elif status == "success":
        color = GREEN
        symbol = "✓"
    elif status == "error":
        color = RED
        symbol = "✗"
    else:
        color = BLUE
        symbol = "○"

    print(f"{color}[{current}/{total} {percentage:5.1f}%]{NC} {symbol} {file_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Batch cache scanner for RustAssistant"
    )
    parser.add_argument("repo_path", type=Path, help="Path to repository")
    parser.add_argument("--limit", type=int, help="Limit number of files to process")
    parser.add_argument(
        "--commit", action="store_true", help="Auto-commit cache after scan"
    )
    parser.add_argument(
        "--skip-cached", action="store_true", help="Skip files with existing cache"
    )
    parser.add_argument(
        "--exclude-tests", action="store_true", help="Exclude test files", default=True
    )

    args = parser.parse_args()

    repo_path = args.repo_path.resolve()
    if not repo_path.exists():
        print(f"❌ Error: Repository not found at {repo_path}")
        sys.exit(1)

    # Setup
    rustassistant = get_rustassistant_path()
    project_root = rustassistant.parent.parent.parent
    env = load_env(project_root)

    print("=" * 60)
    print("🚀 RustAssistant Batch Cache Scanner")
    print("=" * 60)
    print(f"📁 Repository: {repo_path}")
    print()

    # Initialize cache if needed
    cache_dir = repo_path / ".rustassistant"
    if not cache_dir.exists():
        print("🔧 Initializing cache structure...")
        subprocess.run(
            [str(rustassistant), "cache", "init", "--path", str(repo_path)], env=env
        )
        print()

    # Find files
    print("🔍 Scanning for Rust files...")
    exclude_patterns = ["/target/", "/.git/"]
    if args.exclude_tests:
        exclude_patterns.extend(["/tests/", "/benches/", "/examples/"])

    rust_files = find_rust_files(repo_path, exclude_patterns)

    if args.limit:
        rust_files = rust_files[: args.limit]

    total = len(rust_files)
    print(f"📊 Found {total} Rust files to analyze")
    print()

    if total == 0:
        print("⚠️  No files to analyze")
        return

    # Process files
    print("⚙️  Processing files...")
    print()

    stats = {"success": 0, "cached": 0, "failed": 0, "errors": []}

    for idx, file_path in enumerate(rust_files, 1):
        success, cached, message = analyze_file(
            rustassistant, repo_path, file_path, env
        )

        if success:
            if cached:
                stats["cached"] += 1
                print_progress(idx, total, file_path, "cached", True)
            else:
                stats["success"] += 1
                print_progress(idx, total, file_path, "success", False)
        else:
            stats["failed"] += 1
            stats["errors"].append((file_path, message))
            print_progress(idx, total, file_path, "error", False)
            print(f"   └─ Error: {message}")

    # Summary
    print()
    print("=" * 60)
    print("📊 Scan Complete")
    print("=" * 60)
    print()

    # Show cache status
    subprocess.run(
        [str(rustassistant), "cache", "status", "--path", str(repo_path)], env=env
    )

    print()
    print("📈 Statistics:")
    print(f"   Total files:     {total}")
    print(f"   New analyses:    {stats['success']}")
    print(f"   Cache hits:      {stats['cached']}")
    print(f"   Failed:          {stats['failed']}")
    print()

    if stats["errors"]:
        print("❌ Errors:")
        for file_path, error in stats["errors"][:10]:  # Show first 10 errors
            print(f"   • {file_path}: {error}")
        if len(stats["errors"]) > 10:
            print(f"   ... and {len(stats['errors']) - 10} more errors")
        print()

    # Commit if requested
    if args.commit and stats["success"] > 0:
        print("💾 Committing cache files...")
        os.chdir(repo_path)

        # Check if it's a git repo
        result = subprocess.run(["git", "rev-parse", "--git-dir"], capture_output=True)
        if result.returncode == 0:
            subprocess.run(["git", "add", ".rustassistant/"])

            # Check if there are changes
            result = subprocess.run(["git", "diff", "--staged", "--quiet"])
            if result.returncode != 0:
                commit_msg = f"""chore: update rustassistant cache

- Scanned {total} Rust files
- Cached {stats["success"]} new analyses
- {stats["failed"]} errors encountered
- Generated by batch_cache.py"""

                subprocess.run(["git", "commit", "-m", commit_msg])
                print("✅ Cache committed")
            else:
                print("ℹ️  No changes to commit")
        else:
            print("⚠️  Not a git repository, skipping commit")

    # Exit code
    if stats["failed"] > 0:
        sys.exit(1)
    else:
        print("✅ All files processed successfully!")
        sys.exit(0)


if __name__ == "__main__":
    main()
