# Web UI Quick Reference Card

## 🚀 Quick Start
```bash
# Start services
docker compose up -d

# Open Web UI
open http://localhost:3001
```

## 📍 URLs
| Service | URL |
|---------|-----|
| Web UI | http://localhost:3001 |
| Dashboard | http://localhost:3001/dashboard |
| Repositories | http://localhost:3001/repos |
| Queue | http://localhost:3001/queue |
| API | http://localhost:3000 |
| Health Check | http://localhost:3001/health |

## 🔧 Common Tasks

### Add Repository
1. Navigate to `/repos`
2. Click "Add Repository"
3. Enter path and name
4. Submit

**CLI Alternative:**
```bash
curl -X POST http://localhost:3000/api/repos \
  -H "Content-Type: application/json" \
  -d '{"path":"/path/to/repo","name":"myrepo"}'
```

### Enable Auto-Scan
1. Go to `/repos`
2. Find repository
3. Click "Toggle Scan"
4. Verify badge shows "Enabled"

**DB Alternative:**
```bash
sqlite3 data/rustassistant.db \
  "UPDATE repositories SET auto_scan_enabled = 1 WHERE name = 'myrepo';"
```

### Copy Issue to IDE
1. Navigate to `/queue`
2. Find issue
3. Click "📋 Copy for IDE"
4. Paste in IDE (Ctrl+V / Cmd+V)
5. Use with Cursor/Copilot

### Delete Queue Item
1. Go to `/queue`
2. Click "Delete" on item
3. Confirms immediately

### Check Auto-Scanner Status
```bash
# View logs
docker compose logs -f web | grep auto_scanner

# Check enabled repos
curl http://localhost:3000/api/repos | jq '.[] | select(.auto_scan_enabled==1)'
```

## ⚙️ Configuration

### Environment Variables
```bash
# Auto-Scanner
AUTO_SCAN_ENABLED=true
AUTO_SCAN_INTERVAL=60
AUTO_SCAN_MAX_CONCURRENT=2

# Server
HOST=0.0.0.0
PORT=3001
DATABASE_URL=sqlite:/app/data/rustassistant.db
```

### Change Scan Interval
```bash
# Method 1: Environment variable (default for new repos)
AUTO_SCAN_INTERVAL=30 docker compose up -d

# Method 2: Per-repository (database)
sqlite3 data/rustassistant.db \
  "UPDATE repositories SET scan_interval_minutes = 30 WHERE name = 'myrepo';"
```

## 🔍 Monitoring

### Check Services
```bash
docker compose ps
```

### View Logs
```bash
# All services
docker compose logs

# Web UI only
docker compose logs -f web

# Auto-scanner events
docker compose logs -f web | grep -i scan
```

### Database Stats
```sql
-- Connect
sqlite3 data/rustassistant.db

-- Repository stats
SELECT name, auto_scan_enabled, scan_interval_minutes, last_scan_check 
FROM repositories;

-- Queue stats
SELECT stage, COUNT(*) 
FROM queue 
GROUP BY stage;
```

## 🐛 Troubleshooting

### Service Won't Start
```bash
docker compose down
docker compose up -d
docker compose logs web
```

### Database Locked
```bash
docker compose down
rm data/rustassistant.db-wal
rm data/rustassistant.db-shm
docker compose up -d
```

### Auto-Scanner Not Working
```bash
# Check environment
docker compose exec web env | grep AUTO_SCAN

# Force scan check (set last_scan_check to NULL)
sqlite3 data/rustassistant.db \
  "UPDATE repositories SET last_scan_check = NULL WHERE name = 'myrepo';"
```

### Port Already in Use
```bash
# Change port in docker-compose.yml
ports:
  - "3002:3001"  # Changed from 3001

# Or in .env
PORT=3002
```

## 📊 Dashboard Stats

| Stat | Meaning |
|------|---------|
| Total Repos | All repositories tracked |
| Auto-Scan Enabled | Repos with scanning active |
| Queue Pending | Issues waiting for processing |
| Queue Processing | Issues being analyzed |
| Queue Completed | Successfully analyzed |
| Queue Failed | Failed analysis (needs review) |

## 🎯 Workflow Examples

### Daily Review Workflow
```
1. Open dashboard → Check stats
2. Navigate to queue → Review pending items
3. Copy high-priority items → Fix with AI
4. Delete completed items → Clean queue
5. Check repository status → Verify scans running
```

### New Project Setup
```
1. Add repository → /repos/add
2. Enable auto-scan → Toggle on
3. Set interval (optional) → DB update
4. Monitor first scan → Check logs
5. Review issues → /queue
```

### Bulk Management
```bash
# Enable all repos
sqlite3 data/rustassistant.db \
  "UPDATE repositories SET auto_scan_enabled = 1;"

# Disable all repos
sqlite3 data/rustassistant.db \
  "UPDATE repositories SET auto_scan_enabled = 0;"

# Clear completed queue items
sqlite3 data/rustassistant.db \
  "DELETE FROM queue WHERE stage = 'completed';"
```

## 🔗 API Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/health` | Health check |
| GET | `/api/repos` | List repositories |
| POST | `/api/repos` | Add repository |
| DELETE | `/api/repos/:id` | Delete repository |
| GET | `/api/queue/status` | Queue statistics |

## 💡 Pro Tips

1. **Set shorter intervals for active projects** (15-30 min)
2. **Use high priority for critical repos**
3. **Check queue daily** for new issues
4. **Export queue to markdown** for documentation
5. **Monitor auto-scanner logs** during first setup
6. **Copy multiple issues** before fixing batch
7. **Use API for automation** and scripting

## 📱 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+C | Copy selected (future) |
| F5 | Refresh page |
| Ctrl+K | Quick add (future) |

## 🔐 Security Notes

- Web UI has **no authentication** by default
- Use firewall rules to restrict access
- Run behind reverse proxy for production
- Keep repository mounts **read-only**
- Review queue items before copying untrusted content

## 📦 Backup

```bash
# Backup database
cp data/rustassistant.db backup/rustassistant-$(date +%Y%m%d).db

# Backup entire data directory
tar -czf backup/data-$(date +%Y%m%d).tar.gz data/

# Restore
cp backup/rustassistant-20240115.db data/rustassistant.db
```

## 🆘 Emergency Commands

```bash
# Stop everything
docker compose down

# Remove all data (DANGEROUS!)
rm -rf data/
docker compose up -d  # Recreates fresh DB

# Restart just web service
docker compose restart web

# Reset auto-scanner
docker compose exec web pkill -f rustassistant-server
docker compose restart web
```

## 📚 More Help

- Full docs: [WEB_UI_STATUS.md](./WEB_UI_STATUS.md)
- Quick start: [WEB_UI_QUICKSTART.md](./WEB_UI_QUICKSTART.md)
- Auto-scanner: [AUTO_SCANNER_SETUP.md](./AUTO_SCANNER_SETUP.md)

---

**Version**: 0.2.0  
**Last Updated**: 2024-01-15