#!/usr/bin/env python3
"""
RustAssistant Repository and Task Manager

Advanced repository management with task generation and analysis capabilities.
Helps manage multiple repositories, enable scanning, and generate actionable tasks.
"""

import json
import os
import sqlite3
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Configuration
DEFAULT_DB_PATH = "./data/rustassistant.db"
DEFAULT_GITHUB_BASE = "/home/jordan/github"


# ANSI Colors
class Colors:
    RED = "\033[0;31m"
    GREEN = "\033[0;32m"
    YELLOW = "\033[1;33m"
    BLUE = "\033[0;34m"
    MAGENTA = "\033[0;35m"
    CYAN = "\033[0;36m"
    BOLD = "\033[1m"
    NC = "\033[0m"


def log_info(msg: str):
    print(f"{Colors.BLUE}[INFO]{Colors.NC} {msg}")


def log_success(msg: str):
    print(f"{Colors.GREEN}[✓]{Colors.NC} {msg}")


def log_warning(msg: str):
    print(f"{Colors.YELLOW}[⚠]{Colors.NC} {msg}")


def log_error(msg: str):
    print(f"{Colors.RED}[✗]{Colors.NC} {msg}")


def log_header(msg: str):
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'=' * 60}{Colors.NC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{msg:^60}{Colors.NC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'=' * 60}{Colors.NC}\n")


class RepoManager:
    """Manages repositories in RustAssistant database"""

    def __init__(self, db_path: str = DEFAULT_DB_PATH):
        self.db_path = db_path
        if not os.path.exists(db_path):
            log_error(f"Database not found: {db_path}")
            sys.exit(1)
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row

    def __del__(self):
        if hasattr(self, "conn"):
            self.conn.close()

    def list_repos(self, active_only: bool = False) -> List[Dict]:
        """List all repositories"""
        query = """
            SELECT
                id, name, path, status, auto_scan_enabled,
                scan_interval_minutes, last_scan_check, last_analyzed
            FROM repositories
            WHERE path LIKE '/home/%' OR path LIKE '~/%'
        """
        if active_only:
            query += " AND status = 'active'"
        query += " ORDER BY name"

        cursor = self.conn.execute(query)
        return [dict(row) for row in cursor.fetchall()]

    def add_repo(self, path: str, name: str) -> bool:
        """Add a new repository"""
        import uuid

        # Check if path exists
        if not os.path.exists(path):
            log_warning(f"Path does not exist: {path}")
            response = input("Add anyway? (y/n): ").lower()
            if response != "y":
                log_info("Skipping...")
                return False

        # Check for duplicates
        cursor = self.conn.execute(
            "SELECT COUNT(*) as cnt FROM repositories WHERE path = ?", (path,)
        )
        if cursor.fetchone()["cnt"] > 0:
            log_warning(f"Repository already exists: {name}")
            return False

        repo_id = str(uuid.uuid4())
        now = int(datetime.now().timestamp())

        try:
            self.conn.execute(
                """
                INSERT INTO repositories
                (id, path, name, status, created_at, updated_at, auto_scan_enabled, scan_interval_minutes)
                VALUES (?, ?, ?, 'active', ?, ?, 0, 60)
            """,
                (repo_id, path, name, now, now),
            )
            self.conn.commit()
            log_success(f"Added repository: {name}")
            return True
        except Exception as e:
            log_error(f"Failed to add repository: {e}")
            return False

    def enable_scan(self, name: str, interval: int = 60) -> bool:
        """Enable auto-scan for a repository"""
        now = int(datetime.now().timestamp())

        try:
            cursor = self.conn.execute(
                """
                UPDATE repositories
                SET auto_scan_enabled = 1,
                    scan_interval_minutes = ?,
                    updated_at = ?
                WHERE name = ?
            """,
                (interval, now, name),
            )
            self.conn.commit()

            if cursor.rowcount > 0:
                log_success(f"Enabled auto-scan for {name} (interval: {interval}m)")
                return True
            else:
                log_error(f"Repository not found: {name}")
                return False
        except Exception as e:
            log_error(f"Failed to enable scan: {e}")
            return False

    def disable_scan(self, name: str) -> bool:
        """Disable auto-scan for a repository"""
        now = int(datetime.now().timestamp())

        try:
            cursor = self.conn.execute(
                """
                UPDATE repositories
                SET auto_scan_enabled = 0,
                    updated_at = ?
                WHERE name = ?
            """,
                (now, name),
            )
            self.conn.commit()

            if cursor.rowcount > 0:
                log_success(f"Disabled auto-scan for {name}")
                return True
            else:
                log_error(f"Repository not found: {name}")
                return False
        except Exception as e:
            log_error(f"Failed to disable scan: {e}")
            return False

    def cleanup_invalid_repos(self) -> int:
        """Remove repositories with invalid paths (URLs, etc)"""
        try:
            cursor = self.conn.execute(
                "DELETE FROM repositories WHERE path LIKE 'http%'"
            )
            self.conn.commit()
            count = cursor.rowcount
            if count > 0:
                log_success(f"Removed {count} invalid repositories")
            return count
        except Exception as e:
            log_error(f"Failed to cleanup: {e}")
            return 0

    def get_stats(self) -> Dict:
        """Get repository statistics"""
        cursor = self.conn.execute("""
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN auto_scan_enabled = 1 THEN 1 ELSE 0 END) as scanning,
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active
            FROM repositories
            WHERE path LIKE '/home/%' OR path LIKE '~/%'
        """)
        return dict(cursor.fetchone())

    def get_queue_items(self, limit: int = 50) -> List[Dict]:
        """Get items from the queue"""
        cursor = self.conn.execute(
            """
            SELECT
                id, title, description, priority, stage,
                repo_path, file_path, created_at
            FROM queue
            ORDER BY priority, created_at
            LIMIT ?
        """,
            (limit,),
        )
        return [dict(row) for row in cursor.fetchall()]

    def bulk_add_from_directory(self, base_path: str = DEFAULT_GITHUB_BASE) -> int:
        """Bulk add repositories from a directory"""
        if not os.path.exists(base_path):
            log_error(f"Directory not found: {base_path}")
            return 0

        added = 0
        for item in os.listdir(base_path):
            full_path = os.path.join(base_path, item)
            if os.path.isdir(full_path):
                if self.add_repo(full_path, item):
                    added += 1

        return added

    def generate_task_summary(self) -> str:
        """Generate a summary of tasks for AI processing"""
        queue = self.get_queue_items(20)

        if not queue:
            return "No tasks in queue."

        summary = "# Current Tasks in Queue\n\n"

        for item in queue:
            priority_label = ["CRITICAL", "HIGH", "MEDIUM", "LOW"][
                min(item["priority"] - 1, 3)
            ]
            summary += f"## [{priority_label}] {item['title']}\n"
            summary += f"- **Stage:** {item['stage']}\n"
            if item["repo_path"]:
                summary += f"- **Repository:** {item['repo_path']}\n"
            if item["file_path"]:
                summary += f"- **File:** {item['file_path']}\n"
            if item["description"]:
                summary += f"- **Description:** {item['description']}\n"
            summary += "\n"

        return summary


def print_repos_table(repos: List[Dict]):
    """Pretty print repositories table"""
    if not repos:
        log_warning("No repositories found")
        return

    print(
        f"\n{Colors.BOLD}{'Name':<25} {'Scan':<6} {'Interval':<10} {'Status':<10}{Colors.NC}"
    )
    print("-" * 60)

    for repo in repos:
        scan = "✓" if repo["auto_scan_enabled"] else "✗"
        interval = (
            f"{repo['scan_interval_minutes']}m" if repo["auto_scan_enabled"] else "-"
        )
        status = repo["status"]

        # Color code based on scan status
        color = Colors.GREEN if repo["auto_scan_enabled"] else Colors.NC
        print(
            f"{color}{repo['name']:<25} {scan:<6} {interval:<10} {status:<10}{Colors.NC}"
        )


def print_stats(stats: Dict):
    """Pretty print statistics"""
    log_header("Repository Statistics")
    print(f"  Total Repositories:    {Colors.BOLD}{stats['total']}{Colors.NC}")
    print(f"  Active:                {Colors.GREEN}{stats['active']}{Colors.NC}")
    print(f"  Auto-scanning:         {Colors.CYAN}{stats['scanning']}{Colors.NC}")


def print_queue(queue_items: List[Dict]):
    """Pretty print queue items"""
    if not queue_items:
        log_warning("Queue is empty")
        return

    log_header(f"Task Queue ({len(queue_items)} items)")

    for idx, item in enumerate(queue_items, 1):
        priority_labels = {1: "CRITICAL", 2: "HIGH", 3: "MEDIUM", 4: "LOW"}
        priority_colors = {
            1: Colors.RED,
            2: Colors.YELLOW,
            3: Colors.BLUE,
            4: Colors.NC,
        }

        priority = priority_labels.get(item["priority"], "UNKNOWN")
        color = priority_colors.get(item["priority"], Colors.NC)

        print(f"{idx}. {color}[{priority}]{Colors.NC} {item['title']}")
        if item["file_path"]:
            print(f"   📁 {item['file_path']}")


def setup_priority_repos(manager: RepoManager):
    """Setup recommended priority repositories"""
    log_header("Setting Up Priority Repositories")

    priority_repos = {
        "rustscape": {"path": "/home/jordan/github/rustscape", "interval": 30},
        "actions": {"path": "/home/jordan/github/actions", "interval": 60},
        "scripts": {"path": "/home/jordan/github/scripts", "interval": 60},
        "servers_sullivan": {
            "path": "/home/jordan/github/servers_sullivan",
            "interval": 120,
        },
        "servers_freddy": {
            "path": "/home/jordan/github/servers_freddy",
            "interval": 120,
        },
        "fks": {"path": "/home/jordan/github/fks", "interval": 30},
        "rustassistant": {"path": "/home/jordan/github/rustassistant", "interval": 15},
    }

    for name, config in priority_repos.items():
        # Add if not exists
        manager.add_repo(config["path"], name)
        # Enable scanning
        manager.enable_scan(name, config["interval"])


def interactive_menu(manager: RepoManager):
    """Interactive menu for repository management"""
    while True:
        log_header("RustAssistant Repository Manager")
        print("1. List repositories")
        print("2. Add repository")
        print("3. Enable auto-scan")
        print("4. Disable auto-scan")
        print("5. Bulk add from GitHub directory")
        print("6. Setup priority repositories (recommended)")
        print("7. Show statistics")
        print("8. Show task queue")
        print("9. Generate task summary")
        print("10. Cleanup invalid repos")
        print("0. Exit")
        print()

        choice = input("Select option: ").strip()

        try:
            if choice == "1":
                repos = manager.list_repos()
                print_repos_table(repos)

            elif choice == "2":
                path = input("Repository path: ").strip()
                name = input("Repository name: ").strip()
                manager.add_repo(path, name)

            elif choice == "3":
                name = input("Repository name: ").strip()
                interval = input("Scan interval (minutes) [60]: ").strip() or "60"
                manager.enable_scan(name, int(interval))

            elif choice == "4":
                name = input("Repository name: ").strip()
                manager.disable_scan(name)

            elif choice == "5":
                base_path = (
                    input(f"Base path [{DEFAULT_GITHUB_BASE}]: ").strip()
                    or DEFAULT_GITHUB_BASE
                )
                added = manager.bulk_add_from_directory(base_path)
                log_success(f"Added {added} new repositories")

            elif choice == "6":
                setup_priority_repos(manager)

            elif choice == "7":
                stats = manager.get_stats()
                print_stats(stats)

            elif choice == "8":
                queue = manager.get_queue_items()
                print_queue(queue)

            elif choice == "9":
                summary = manager.generate_task_summary()
                print(f"\n{summary}")

                save = input("\nSave to file? (y/n): ").lower()
                if save == "y":
                    filename = (
                        f"task_summary_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
                    )
                    with open(filename, "w") as f:
                        f.write(summary)
                    log_success(f"Saved to {filename}")

            elif choice == "10":
                manager.cleanup_invalid_repos()

            elif choice == "0":
                log_info("Goodbye!")
                break

            else:
                log_error("Invalid option")

        except KeyboardInterrupt:
            print()
            log_info("Interrupted. Exiting...")
            break
        except Exception as e:
            log_error(f"Error: {e}")

        input("\nPress Enter to continue...")


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="RustAssistant Repository Manager")
    parser.add_argument("--db", default=DEFAULT_DB_PATH, help="Database path")
    parser.add_argument(
        "command",
        nargs="?",
        default="interactive",
        choices=[
            "list",
            "add",
            "enable",
            "disable",
            "bulk",
            "priority",
            "stats",
            "queue",
            "summary",
            "cleanup",
            "interactive",
        ],
        help="Command to execute",
    )
    parser.add_argument("--name", help="Repository name")
    parser.add_argument("--path", help="Repository path")
    parser.add_argument(
        "--interval", type=int, default=60, help="Scan interval in minutes"
    )
    parser.add_argument(
        "--base", default=DEFAULT_GITHUB_BASE, help="Base directory for bulk add"
    )

    args = parser.parse_args()

    manager = RepoManager(args.db)

    if args.command == "list":
        repos = manager.list_repos()
        print_repos_table(repos)

    elif args.command == "add":
        if not args.path or not args.name:
            log_error("--path and --name are required for add command")
            sys.exit(1)
        manager.add_repo(args.path, args.name)

    elif args.command == "enable":
        if not args.name:
            log_error("--name is required for enable command")
            sys.exit(1)
        manager.enable_scan(args.name, args.interval)

    elif args.command == "disable":
        if not args.name:
            log_error("--name is required for disable command")
            sys.exit(1)
        manager.disable_scan(args.name)

    elif args.command == "bulk":
        added = manager.bulk_add_from_directory(args.base)
        log_success(f"Added {added} new repositories")

    elif args.command == "priority":
        setup_priority_repos(manager)
        stats = manager.get_stats()
        print_stats(stats)

    elif args.command == "stats":
        stats = manager.get_stats()
        print_stats(stats)

    elif args.command == "queue":
        queue = manager.get_queue_items()
        print_queue(queue)

    elif args.command == "summary":
        summary = manager.generate_task_summary()
        print(summary)

    elif args.command == "cleanup":
        manager.cleanup_invalid_repos()

    elif args.command == "interactive":
        interactive_menu(manager)


if __name__ == "__main__":
    main()
