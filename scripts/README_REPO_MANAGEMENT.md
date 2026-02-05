# Repository Management Scripts

This directory contains scripts to help you manage repositories and tasks in RustAssistant.

## Quick Start

### One-Command Setup (Recommended)

Set up all your priority repositories with auto-scanning enabled:

```bash
./scripts/quick_setup.sh
```

This will:
- ✅ Add rustscape, actions, scripts, servers_sullivan, servers_freddy
- ✅ Enable auto-scanning with optimized intervals
- ✅ Clean up any invalid repository entries
- ✅ Show you a summary of configured repositories

### Interactive Management

For ongoing management, use the interactive menu:

```bash
# Bash version (simpler, faster)
./scripts/manage_repos.sh interactive

# Python version (more features)
python3 scripts/repo_manager.py
```

---

## Scripts Overview

### 1. `quick_setup.sh` - One-Command Setup

**Best for:** Initial setup

**What it does:**
- Adds your priority repositories
- Enables auto-scanning with recommended intervals
- Cleans up invalid entries

**Usage:**
```bash
./scripts/quick_setup.sh
```

**Repositories configured:**
- `rustscape` - 30 min intervals (active development)
- `actions` - 60 min intervals (shared actions)
- `scripts` - 60 min intervals (shared scripts)
- `servers_sullivan` - 120 min intervals (home server config)
- `servers_freddy` - 120 min intervals (home server config)
- `rustassistant` - 15 min intervals (this project)
- `fks` - 30 min intervals (if exists)

---

### 2. `manage_repos.sh` - Bash Management Tool

**Best for:** Quick operations, scripting, lightweight management

**Features:**
- List repositories
- Add/remove repositories
- Enable/disable auto-scanning
- Bulk operations
- Statistics and queue viewing

**Usage:**

```bash
# Show help
./scripts/manage_repos.sh help

# List all repositories
./scripts/manage_repos.sh list

# Add a repository
./scripts/manage_repos.sh add /path/to/repo repo_name

# Enable scanning (with 30 min interval)
./scripts/manage_repos.sh enable rustscape 30

# Disable scanning
./scripts/manage_repos.sh disable rustscape

# Bulk add all repos from GitHub directory
./scripts/manage_repos.sh bulk

# Enable priority repos
./scripts/manage_repos.sh priority

# Show statistics
./scripts/manage_repos.sh stats

# Show task queue
./scripts/manage_repos.sh queue

# Clean up invalid repos
./scripts/manage_repos.sh cleanup

# Interactive mode
./scripts/manage_repos.sh interactive
```

**Environment Variables:**
```bash
# Custom database path
DB_PATH=./custom/path/db.sqlite ./scripts/manage_repos.sh list

# Custom GitHub directory
GITHUB_BASE=/custom/github ./scripts/manage_repos.sh bulk
```

---

### 3. `repo_manager.py` - Python Management Tool

**Best for:** Advanced features, task generation, detailed reporting

**Features:**
- Everything in manage_repos.sh, plus:
- Task summary generation
- Pretty formatted output
- Advanced filtering
- Export capabilities

**Usage:**

```bash
# Show help
python3 scripts/repo_manager.py --help

# List repositories
python3 scripts/repo_manager.py list

# Add repository
python3 scripts/repo_manager.py add --path /path/to/repo --name repo_name

# Enable scanning
python3 scripts/repo_manager.py enable --name rustscape --interval 30

# Disable scanning
python3 scripts/repo_manager.py disable --name rustscape

# Bulk add
python3 scripts/repo_manager.py bulk --base /home/jordan/github

# Setup priority repos
python3 scripts/repo_manager.py priority

# Show statistics
python3 scripts/repo_manager.py stats

# Show queue
python3 scripts/repo_manager.py queue

# Generate task summary
python3 scripts/repo_manager.py summary

# Clean up
python3 scripts/repo_manager.py cleanup

# Interactive mode (recommended)
python3 scripts/repo_manager.py
python3 scripts/repo_manager.py interactive
```

**Advanced Options:**
```bash
# Custom database
python3 scripts/repo_manager.py --db ./custom.db list

# Custom base directory for bulk add
python3 scripts/repo_manager.py bulk --base /different/path

# Custom scan interval
python3 scripts/repo_manager.py enable --name myrepo --interval 15
```

---

## Scan Interval Recommendations

Choose scan intervals based on how actively you're developing:

| Interval | Use Case | Examples |
|----------|----------|----------|
| **15 min** | Active development, frequent changes | rustassistant (this project) |
| **30 min** | Regular development | rustscape, fks |
| **60 min** | Moderate activity | actions, scripts |
| **120 min** | Configuration files, infrequent changes | servers_sullivan, servers_freddy |
| **240 min** | Archive/reference repos | tools, games |

**Note:** Shorter intervals mean more frequent scans and faster detection of issues, but also more resource usage.

---

## Workflows

### Initial Setup

```bash
# 1. Run quick setup
./scripts/quick_setup.sh

# 2. Restart RustAssistant to apply changes
docker compose restart rustassistant

# 3. Open Web UI to verify
open http://localhost:3001/repos

# 4. Make a test change in a monitored repo
echo "# Test" >> /home/jordan/github/rustscape/README.md

# 5. Wait for scan interval, then check queue
open http://localhost:3001/queue
```

### Daily Workflow

```bash
# 1. Check queue for new tasks
python3 scripts/repo_manager.py queue

# 2. Generate task summary for AI
python3 scripts/repo_manager.py summary > tasks_today.md

# 3. Copy tasks to your IDE/AI assistant
cat tasks_today.md

# 4. Check repository status
python3 scripts/repo_manager.py stats
```

### Adding New Repositories

```bash
# Option 1: Interactive
./scripts/manage_repos.sh interactive
# Then choose option 2 (Add repository)

# Option 2: Command line
./scripts/manage_repos.sh add /home/jordan/github/newrepo newrepo

# Option 3: Enable scanning immediately
./scripts/manage_repos.sh add /home/jordan/github/newrepo newrepo
./scripts/manage_repos.sh enable newrepo 60

# Option 4: Python version
python3 scripts/repo_manager.py add --path /home/jordan/github/newrepo --name newrepo
python3 scripts/repo_manager.py enable --name newrepo --interval 60
```

### Bulk Operations

```bash
# Add all repos from GitHub directory
./scripts/manage_repos.sh bulk

# Then selectively enable scanning
./scripts/manage_repos.sh enable repo1 30
./scripts/manage_repos.sh enable repo2 60
./scripts/manage_repos.sh enable repo3 120

# Or enable all priority repos at once
./scripts/manage_repos.sh priority
```

### Cleaning Up

```bash
# Remove repos with invalid paths (URLs, etc)
./scripts/manage_repos.sh cleanup

# Or use Python version
python3 scripts/repo_manager.py cleanup
```

---

## Integration with Web UI

All these scripts modify the same database as the Web UI. Changes are reflected immediately:

1. **Add/modify repos via scripts** → See changes at `http://localhost:3001/repos`
2. **Enable scanning via scripts** → Auto-scanner picks up changes immediately
3. **View queue via scripts or UI** → Both show the same data

**Best Practice:** Use scripts for bulk operations, use Web UI for quick toggles and monitoring.

---

## Troubleshooting

### Database not found

```bash
# Make sure you're in the rustassistant directory
cd /home/jordan/github/rustassistant

# Run setup
./scripts/quick_setup.sh
```

### Repository path issues

```bash
# Use absolute paths
./scripts/manage_repos.sh add /home/jordan/github/myrepo myrepo

# NOT relative paths like ../myrepo or ~/github/myrepo
```

### Scanning not working

```bash
# 1. Verify repo is added and enabled
./scripts/manage_repos.sh list

# 2. Check auto-scanner is running
docker compose logs rustassistant | grep -i "auto-scan"

# 3. Restart the service
docker compose restart rustassistant

# 4. Check environment variables
docker compose exec rustassistant env | grep AUTO_SCAN
```

### UUID errors on BSD/macOS

The scripts use `/proc/sys/kernel/random/uuid` which is Linux-only. If you're on macOS:

```bash
# Install uuidgen (usually pre-installed)
which uuidgen

# Or modify scripts to use Python
python3 -c "import uuid; print(uuid.uuid4())"
```

---

## Examples

### Example 1: Setup for RuneScape Development

```bash
# Add rustscape with aggressive scanning
./scripts/manage_repos.sh add /home/jordan/github/rustscape rustscape
./scripts/manage_repos.sh enable rustscape 15

# Verify
./scripts/manage_repos.sh list | grep rustscape
```

### Example 2: Setup Home Server Monitoring

```bash
# Add both servers with relaxed scanning
./scripts/manage_repos.sh add /home/jordan/github/servers_sullivan servers_sullivan
./scripts/manage_repos.sh add /home/jordan/github/servers_freddy servers_freddy

./scripts/manage_repos.sh enable servers_sullivan 180
./scripts/manage_repos.sh enable servers_freddy 180
```

### Example 3: Weekly Task Review

```bash
# Generate comprehensive task summary
python3 scripts/repo_manager.py summary > weekly_tasks_$(date +%Y%m%d).md

# Review statistics
python3 scripts/repo_manager.py stats

# Check queue
python3 scripts/repo_manager.py queue
```

### Example 4: Add All GitHub Repos, Enable Priority Ones

```bash
# Step 1: Bulk add everything
./scripts/manage_repos.sh bulk

# Step 2: Enable scanning for active projects only
./scripts/manage_repos.sh enable rustscape 30
./scripts/manage_repos.sh enable actions 60
./scripts/manage_repos.sh enable scripts 60
./scripts/manage_repos.sh enable servers_sullivan 120
./scripts/manage_repos.sh enable servers_freddy 120

# Step 3: Review
./scripts/manage_repos.sh stats
```

---

## API Integration

These scripts interact directly with the SQLite database. For API-based interactions:

```bash
# Add repository via API
curl -X POST http://localhost:3001/api/repos \
  -H "Content-Type: application/json" \
  -d '{"path":"/home/jordan/github/myrepo","name":"myrepo"}'

# List repositories via API
curl http://localhost:3001/api/repos

# Get queue via API
curl http://localhost:3001/api/queue
```

---

## Tips & Best Practices

1. **Start with quick_setup.sh** - Gets you up and running in seconds
2. **Use interactive mode for exploration** - Great for learning the system
3. **Use command-line mode for automation** - Script your workflows
4. **Adjust intervals based on activity** - More active repos = shorter intervals
5. **Monitor the queue regularly** - Don't let tasks pile up
6. **Generate summaries for AI sessions** - Copy task lists to your IDE
7. **Clean up periodically** - Remove archived or moved repositories
8. **Use the Web UI for quick toggles** - Scripts for bulk operations

---

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_PATH` | `./data/rustassistant.db` | Path to SQLite database |
| `GITHUB_BASE` | `/home/jordan/github` | Base directory for repositories |
| `AUTO_SCAN_ENABLED` | `true` | Enable/disable auto-scanner |
| `AUTO_SCAN_INTERVAL` | `300` | Default scan check interval (seconds) |
| `AUTO_SCAN_MAX_CONCURRENT` | `3` | Max concurrent scans |

---

## File Permissions

Make scripts executable:

```bash
chmod +x scripts/quick_setup.sh
chmod +x scripts/manage_repos.sh
chmod +x scripts/repo_manager.py
```

---

## Next Steps

After setting up your repositories:

1. ✅ Visit the Web UI at http://localhost:3001
2. ✅ Check the queue at http://localhost:3001/queue
3. ✅ Make a change in a monitored repo to test scanning
4. ✅ Copy queue items to your IDE using "Copy for IDE" button
5. ✅ Review and prioritize tasks in your AI workflow

---

**Need Help?**
- See main [README.md](../README.md)
- Check [Web UI documentation](../docs/user/WEB_UI_STATUS.md)
- Review [Auto-scanner setup](../docs/user/AUTO_SCANNER_SETUP.md)