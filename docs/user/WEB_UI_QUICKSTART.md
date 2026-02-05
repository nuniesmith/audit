# Web UI Quick Start Guide

## 🚀 Get Started in 5 Minutes

Welcome to RustAssistant Web UI! This guide will get you up and running quickly.

---

## Prerequisites

- Docker and Docker Compose installed
- OR Rust toolchain (for local development)
- Your GitHub repositories on disk

---

## Option 1: Docker (Recommended)

### Step 1: Start the Services

```bash
cd /home/jordan/github/rustassistant

# Start all services (API, Web UI, Redis)
docker compose up -d

# Check status
docker compose ps
```

Expected output:
```
NAME                   STATUS              PORTS
rustassistant-api      Up (healthy)        0.0.0.0:3000->3000/tcp
rustassistant-web      Up (healthy)        0.0.0.0:3001->3001/tcp
rustassistant-redis    Up (healthy)        0.0.0.0:6379->6379/tcp
```

### Step 2: Open the Web UI

Open your browser and navigate to: **http://localhost:3001**

You should see the dashboard!

### Step 3: Add Your First Repository

1. Click **"+ Add Repository"** button
2. Fill in the form:
   - **Path**: `/home/jordan/github/fks` (or your repo path)
   - **Name**: `fks` (or a friendly name)
3. Click **"Add Repository"**
4. You'll be redirected to the repository list

### Step 4: Enable Auto-Scanning

1. Find your repository in the list
2. Click **"Toggle Scan"** button
3. The status badge will change to ✅ Enabled (60min)
4. The auto-scanner will check this repo every 60 minutes

### Step 5: Check the Queue

1. Navigate to **Queue** in the top menu
2. Wait for the scanner to detect issues (or manually add some)
3. Use **"📋 Copy for IDE"** to copy issue content
4. Paste into your IDE's AI assistant (Cursor, GitHub Copilot, etc.)

---

## Option 2: Local Development

### Step 1: Build and Run

```bash
cd /home/jordan/github/rustassistant

# Set environment variables
export DATABASE_URL=sqlite:data/rustassistant.db
export AUTO_SCAN_ENABLED=true
export AUTO_SCAN_INTERVAL=60

# Run the server
cargo run --bin rustassistant-server
```

### Step 2: Access the UI

Open: **http://localhost:3001**

---

## 🎯 Main Features

### Dashboard (`/`)
- Real-time statistics
- Repository count
- Queue status overview
- Quick action buttons

### Repositories (`/repos`)
- **List**: View all tracked repositories
- **Add**: Add new repositories via form
- **Toggle**: Enable/disable auto-scanning per repo
- **Delete**: Remove repositories (with confirmation)

### Queue (`/queue`)
- **View**: All queued tasks and issues
- **Copy**: One-click copy to clipboard
- **Delete**: Remove completed items
- **Filter**: By priority, stage, source

---

## 📋 Common Workflows

### Workflow 1: Setup Auto-Scanning for Active Project

```
1. Add repository      → /repos/add
2. Toggle auto-scan    → Click "Toggle Scan"
3. Monitor dashboard   → /dashboard
4. Check queue daily   → /queue
5. Copy & fix issues   → "Copy for IDE" button
```

### Workflow 2: Manual Analysis

```
1. Add repository      → /repos/add
2. Keep auto-scan OFF  → Leave toggle disabled
3. Trigger manual scan → "Scan Now" button
4. Review queue        → /queue
5. Copy issues to IDE  → "Copy for IDE"
```

### Workflow 3: Queue Management

```
1. Open queue page     → /queue
2. Review pending      → Yellow border items
3. Copy to IDE         → Click "Copy for IDE" button
4. Paste in IDE        → Ctrl+V / Cmd+V
5. Fix with AI         → Use Cursor/Copilot
6. Delete from queue   → Click "Delete"
```

---

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# Database
DATABASE_URL=sqlite:data/rustassistant.db

# Server
HOST=0.0.0.0
PORT=3001

# Auto-Scanner
AUTO_SCAN_ENABLED=true
AUTO_SCAN_INTERVAL=60
AUTO_SCAN_MAX_CONCURRENT=2

# Logging
RUST_LOG=info,rustassistant=debug
```

### Docker Compose Override

Create `docker-compose.override.yml`:

```yaml
services:
  web:
    environment:
      - AUTO_SCAN_INTERVAL=30  # Scan every 30 minutes
      - AUTO_SCAN_MAX_CONCURRENT=3
    volumes:
      - /your/custom/path:/repos:ro
```

Then restart:
```bash
docker compose down
docker compose up -d
```

---

## 🐛 Troubleshooting

### Web UI Not Loading

**Problem**: Browser shows "Cannot connect"

**Solution**:
```bash
# Check if container is running
docker compose ps

# Check logs
docker compose logs web

# Restart the service
docker compose restart web
```

### Repository Not Found

**Problem**: "Repository not found" error

**Solution**:
- Ensure the path is absolute (starts with `/`)
- Verify the path exists: `ls -la /home/jordan/github/fks`
- Check Docker volume mounts in `docker-compose.yml`
- Restart container after adding volumes

### Auto-Scanner Not Working

**Problem**: No issues appearing in queue

**Solution**:
```bash
# Check auto-scanner is enabled
docker compose exec web env | grep AUTO_SCAN

# View logs
docker compose logs -f web | grep auto_scanner

# Verify repository has auto-scan enabled
curl http://localhost:3001/api/repos | jq '.[] | select(.name=="fks")'
```

### Database Locked Error

**Problem**: "Database is locked"

**Solution**:
```bash
# Stop all services
docker compose down

# Remove SQLite WAL files
rm data/rustassistant.db-wal
rm data/rustassistant.db-shm

# Restart
docker compose up -d
```

---

## 🎨 UI Tour

### Dashboard Layout
```
┌─────────────────────────────────────────────────────┐
│ 🦀 RustAssistant                                    │
│ Developer Workflow Management System                │
├─────────────────────────────────────────────────────┤
│ [Dashboard] [Repositories] [Queue] [Auto-Scanner]   │
├─────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │
│ │ Total Repos │ │ Auto-Scan   │ │ Queue       │    │
│ │     3       │ │ Enabled: 2  │ │ Pending: 5  │    │
│ └─────────────┘ └─────────────┘ └─────────────┘    │
├─────────────────────────────────────────────────────┤
│ Quick Actions                                       │
│ [+ Add Repository] [View Queue] [Scanner Settings]  │
└─────────────────────────────────────────────────────┘
```

### Repository Card
```
┌─────────────────────────────────────────────────────┐
│ fks                                    ✅ active     │
├─────────────────────────────────────────────────────┤
│ Path: /home/jordan/github/fks                       │
│ Auto-Scan: ✅ Enabled (60min)                       │
│ Last Scan: 2024-01-15 14:30:00                     │
│ Created: 2024-01-10 09:15:00                       │
├─────────────────────────────────────────────────────┤
│ [Toggle Scan] [Scan Now] [Configure] [Delete]      │
└─────────────────────────────────────────────────────┘
```

### Queue Item
```
┌─────────────────────────────────────────────────────┐
│ abc123de     [PENDING]     [HIGH]                   │
├─────────────────────────────────────────────────────┤
│ Fix clippy warning in auto_scanner.rs:              │
│ Line 245: Unnecessary cast from i64 to i64          │
│                                                     │
│ Suggested fix:                                      │
│ Remove the `as i64` cast since the value is        │
│ already of type i64                                │
├─────────────────────────────────────────────────────┤
│ Source: auto-scan  Created: 5 mins ago              │
│ [📋 Copy for IDE] [Delete]                          │
└─────────────────────────────────────────────────────┘
```

---

## 🔗 Keyboard Shortcuts (Future)

Coming soon:
- `Ctrl+K` - Quick add repository
- `Ctrl+Q` - Go to queue
- `Ctrl+D` - Go to dashboard
- `Ctrl+C` - Copy selected item

---

## 💡 Pro Tips

### 1. Use Custom Scan Intervals
For active projects, use shorter intervals:
```bash
# Edit database directly
sqlite3 data/rustassistant.db "UPDATE repositories SET scan_interval_minutes = 30 WHERE name = 'myproject';"
```

### 2. Bulk Operations
Use the API for bulk operations:
```bash
# Enable auto-scan for all repos
curl -X POST http://localhost:3000/api/repos/bulk/enable-scan
```

### 3. Export Queue to Markdown
```bash
# Get queue as JSON
curl http://localhost:3000/api/queue/status | jq -r '.items[] | "## \(.id)\n\(.content)\n"' > issues.md
```

### 4. Monitor Auto-Scanner
```bash
# Watch scanner logs in real-time
docker compose logs -f web | grep -i "auto.*scan"
```

### 5. Copy Multiple Items
1. Open queue in multiple tabs
2. Copy items to different clipboard managers
3. Or use the API to batch export

---

## 🚦 Next Steps

Now that you're set up:

1. ✅ Add all your active repositories
2. ✅ Enable auto-scan for projects you're working on
3. ✅ Set up a daily routine to check the queue
4. ✅ Integrate with your IDE's AI assistant
5. ✅ Monitor the dashboard for overview

---

## 📚 Additional Resources

- [Full Web UI Documentation](./WEB_UI_STATUS.md)
- [Auto-Scanner Setup](./AUTO_SCANNER_SETUP.md)
- [Docker Deployment Guide](./DEPLOYMENT_SUCCESS.md)
- [API Documentation](http://localhost:3001/)

---

## 🤝 Getting Help

### Check Logs
```bash
# All services
docker compose logs

# Specific service
docker compose logs web

# Follow live
docker compose logs -f web
```

### Health Checks
```bash
# Web UI
curl http://localhost:3001/health

# API
curl http://localhost:3000/health

# Redis
docker compose exec redis redis-cli ping
```

### Database Inspection
```bash
# Connect to SQLite
sqlite3 data/rustassistant.db

# List repositories
sqlite3 data/rustassistant.db "SELECT id, name, auto_scan_enabled FROM repositories;"

# Check queue
sqlite3 data/rustassistant.db "SELECT COUNT(*), stage FROM queue GROUP BY stage;"
```

---

## 🎉 Success!

You're all set! The Web UI is ready to help you manage your code quality and workflow.

**Happy coding!** 🦀

---

**Last Updated**: 2024-01-15  
**Version**: 0.2.0