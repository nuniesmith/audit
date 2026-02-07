# Repository Management Quick Reference

> **✨ Your repositories are now configured!** All priority repos are being monitored with auto-scanning enabled.

## 🎯 Your Configured Repositories

| Repository | Path | Scan Interval | Purpose |
|------------|------|---------------|---------|
| **rustscape** | `/home/jordan/github/rustscape` | 30 min | RuneScape project (personal) |
| **actions** | `/home/jordan/github/actions` | 60 min | Shared GitHub Actions |
| **scripts** | `/home/jordan/github/scripts` | 60 min | Shared scripts |
| **servers_sullivan** | `/home/jordan/github/servers_sullivan` | 120 min | Sullivan home server |
| **servers_freddy** | `/home/jordan/github/servers_freddy` | 120 min | Freddy home server |
| **rustassistant** | `/home/jordan/github/rustassistant` | 15 min | This project |
| **fks** | `/home/jordan/github/fks` | 30 min | FKS project |

## 🚀 Quick Commands

### View Current Status
```bash
# List all repositories
bash scripts/manage_repos.sh list

# Show statistics
bash scripts/manage_repos.sh stats

# Check task queue
bash scripts/manage_repos.sh queue

# Python version (prettier output)
python3 scripts/repo_manager.py stats
python3 scripts/repo_manager.py queue
```

### Web UI (Recommended)
```bash
# Open repositories page
http://localhost:3000/repos

# View task queue
http://localhost:3000/queue

# Dashboard
http://localhost:3000
```

### Manage Repositories

#### Add a New Repository
```bash
# Using bash script
bash scripts/manage_repos.sh add /home/jordan/github/newrepo newrepo

# Using Python script
python3 scripts/repo_manager.py add --path /home/jordan/github/newrepo --name newrepo
```

#### Enable/Disable Auto-Scan
```bash
# Enable with 60-minute interval
bash scripts/manage_repos.sh enable newrepo 60

# Disable
bash scripts/manage_repos.sh disable newrepo

# Python version
python3 scripts/repo_manager.py enable --name newrepo --interval 60
python3 scripts/repo_manager.py disable --name newrepo
```

#### Adjust Scan Interval
```bash
# Change rustscape to scan every 15 minutes
bash scripts/manage_repos.sh enable rustscape 15

# Change servers to scan every 4 hours (240 minutes)
bash scripts/manage_repos.sh enable servers_sullivan 240
bash scripts/manage_repos.sh enable servers_freddy 240
```

## 📊 Workflow Examples

### Daily Task Review
```bash
# 1. Check what tasks are in the queue
python3 scripts/repo_manager.py queue

# 2. Generate task summary for AI assistant
python3 scripts/repo_manager.py summary > daily_tasks.md

# 3. Copy tasks to your IDE (Cursor, etc.)
cat daily_tasks.md

# Or use the Web UI:
# http://localhost:3000/queue
# Click "Copy for IDE" button on any task
```

### Weekly Cleanup
```bash
# 1. Review repository statistics
python3 scripts/repo_manager.py stats

# 2. Clean up invalid entries
bash scripts/manage_repos.sh cleanup

# 3. Check for completed tasks
# (manually review queue via Web UI)
```

### Adding Multiple Repositories
```bash
# Option 1: Interactive mode (easiest)
bash scripts/manage_repos.sh interactive

# Option 2: Bulk add all from GitHub directory
bash scripts/manage_repos.sh bulk

# Option 3: One by one
bash scripts/manage_repos.sh add /home/jordan/github/repo1 repo1
bash scripts/manage_repos.sh add /home/jordan/github/repo2 repo2
bash scripts/manage_repos.sh add /home/jordan/github/repo3 repo3

# Then enable scanning
bash scripts/manage_repos.sh enable repo1 30
bash scripts/manage_repos.sh enable repo2 60
bash scripts/manage_repos.sh enable repo3 120
```

## 🎛️ Scan Interval Guidelines

Choose intervals based on your development activity:

| Interval | When to Use | Examples |
|----------|-------------|----------|
| **15 min** | Active development, constant changes | rustassistant (working on it now) |
| **30 min** | Regular development, frequent commits | rustscape, fks |
| **60 min** | Moderate activity, daily changes | actions, scripts |
| **120 min** | Configuration repos, occasional updates | servers_sullivan, servers_freddy |
| **240+ min** | Archive/reference, rarely changing | tools, old projects |

**💡 Tip:** Shorter intervals = faster issue detection, but more resource usage

## 🔧 Advanced Usage

### Interactive Menu
```bash
# Bash version (fast, simple)
bash scripts/manage_repos.sh interactive

# Python version (more features, prettier)
python3 scripts/repo_manager.py
```

### Custom Database Path
```bash
DB_PATH=/custom/path/db.sqlite bash scripts/manage_repos.sh list
```

### Custom GitHub Directory
```bash
GITHUB_BASE=/different/path bash scripts/manage_repos.sh bulk
```

### Generate Task Summary for AI
```bash
# Generate markdown summary
python3 scripts/repo_manager.py summary

# Save to file with timestamp
python3 scripts/repo_manager.py summary > tasks_$(date +%Y%m%d_%H%M).md
```

## 🐛 Troubleshooting

### Auto-scanning not working?
```bash
# 1. Verify repository is enabled
bash scripts/manage_repos.sh list

# 2. Restart RustAssistant
docker compose restart rustassistant

# 3. Check logs
docker compose logs -f rustassistant | grep -i "auto-scan"

# 4. Verify environment variables
docker compose exec rustassistant env | grep AUTO_SCAN
```

### Database locked errors?
```bash
# Stop RustAssistant before running scripts
docker compose stop rustassistant

# Run your script
bash scripts/manage_repos.sh list

# Restart
docker compose start rustassistant
```

### Repository not being scanned?
```bash
# Check the path is correct (must be absolute)
bash scripts/manage_repos.sh list | grep myrepo

# Verify path exists
ls -la /home/jordan/github/myrepo

# Re-enable with correct path
bash scripts/manage_repos.sh disable myrepo
bash scripts/manage_repos.sh add /home/jordan/github/myrepo myrepo
bash scripts/manage_repos.sh enable myrepo 60
```

## 📝 Next Steps

### 1. Test the Auto-Scanner
```bash
# Make a small change in a monitored repo
echo "# Test change" >> /home/jordan/github/rustscape/README.md
cd /home/jordan/github/rustscape && git add . && git commit -m "test"

# Wait for the scan interval (e.g., 30 minutes for rustscape)
# OR restart to trigger immediate scan
docker compose restart rustassistant

# Check the queue
http://localhost:3000/queue
```

### 2. Integrate with Your IDE
- Open Web UI: http://localhost:3000/queue
- Click "Copy for IDE" on any task
- Paste into your AI assistant (Cursor, Copilot, etc.)
- Let the AI help you fix the issue!

### 3. Customize Your Workflow
```bash
# Create a daily script
cat > ~/bin/check-rustassistant << 'EOF'
#!/bin/bash
echo "📊 RustAssistant Daily Summary"
echo "=============================="
python3 /home/jordan/github/rustassistant/scripts/repo_manager.py stats
echo ""
echo "📋 Current Queue:"
python3 /home/jordan/github/rustassistant/scripts/repo_manager.py queue
EOF

chmod +x ~/bin/check-rustassistant

# Run it daily
crontab -e
# Add: 0 9 * * * ~/bin/check-rustassistant
```

## 🔗 Quick Links

- **Web UI Dashboard:** http://localhost:3000
- **Repositories Page:** http://localhost:3000/repos
- **Task Queue:** http://localhost:3000/queue
- **API Health Check:** http://localhost:3000/health
- **API Docs:** http://localhost:3000/api/*

## 📚 Documentation

For more details, see:
- [Scripts README](scripts/README_REPO_MANAGEMENT.md) - Detailed script documentation
- [Web UI Status](docs/user/WEB_UI_STATUS.md) - Web UI features
- [Auto-Scanner Setup](docs/user/AUTO_SCANNER_SETUP.md) - Scanner configuration
- [Main README](README.md) - Full project documentation

## 💡 Pro Tips

1. **Use the Web UI for quick toggles** - Fast and visual
2. **Use scripts for bulk operations** - Efficient for many repos
3. **Generate task summaries weekly** - Keep your AI assistant updated
4. **Adjust intervals seasonally** - Tighten during active dev, relax during maintenance
5. **Monitor the queue regularly** - Don't let tasks pile up
6. **Copy tasks to IDE daily** - Integrate with your AI workflow
7. **Clean up old repos** - Remove archived projects

## 🎉 You're All Set!

Your repositories are now being monitored automatically. The auto-scanner will:
- ✅ Check for changes based on your configured intervals
- ✅ Detect TODOs, FIXMEs, and potential issues
- ✅ Generate actionable tasks in the queue
- ✅ Make tasks available via Web UI and API

**Next:** Make a change in one of your repos and watch the magic happen! 🚀