# Simplified Setup Guide - Two Containers Only

## 🎉 What Changed

RustAssistant now runs with just **TWO containers**:
1. **rustassistant** - Unified server (Web UI + API combined)
2. **redis** - Cache

Previously, we had three containers (api, web, redis). Now the API and Web UI are served from a single container on port 3000.

---

## 🚀 Quick Start

### Start Services
```bash
cd /home/jordan/github/rustassistant

# Stop old containers (if running)
docker compose down

# Start new simplified setup
docker compose up -d

# Check status
docker compose ps
```

Expected output:
```
NAME               STATUS              PORTS
rustassistant      Up (healthy)        0.0.0.0:3000->3000/tcp
rustassistant-redis Up (healthy)        0.0.0.0:6379->6379/tcp
```

### Access Points

| Service | URL | Description |
|---------|-----|-------------|
| Web UI | http://localhost:3000 | Dashboard, repos, queue |
| API | http://localhost:3000/api/* | REST API endpoints |
| Health | http://localhost:3000/health | Health check |

**Note**: Everything is now on port **3000** (previously API was on 3000)

---

## 🔄 Migration from Old Setup

### If You Had the Old 3-Container Setup

```bash
# 1. Stop old containers
docker compose down

# 2. Remove old images (optional, saves disk space)
docker rmi rustassistant-api rustassistant-web

# 3. Start new setup
docker compose up -d

# 4. Verify
curl http://localhost:3000/health
curl http://localhost:3000/api/repos
open http://localhost:3000
```

### Update Your Scripts/Bookmarks

**Old URLs** → **New URLs**
- ~~http://localhost:3000/api/repos~~ → http://localhost:3000/api/repos
- ~~http://localhost:3000/health~~ → http://localhost:3000/health
- http://localhost:3000/ → http://localhost:3000/ (unchanged)

### Update Environment Variables

If you have custom port settings in `.env`:

**Before:**
```bash
PORT=3000  # For API
```

**After:**
```bash
PORT=3000  # For unified server
```

---

## 📦 Container Details

### Rustassistant Container

**Serves:**
- Web UI at root paths (`/`, `/dashboard`, `/repos`, `/queue`)
- REST API at `/api/*`
- Health check at `/health`
- Auto-scanner background task

**Resources:**
- Memory: 512MB-1GB
- CPU: 1-2 cores
- Storage: Shared `./data` directory

**Ports:**
- 3000 (Web UI + API)

**Volumes:**
- `./data` - Database and cache
- `./config` - Configuration files (read-only)
- `/home/jordan/github` - Your repositories (read-only)

### Redis Container

**Purpose:** LLM response caching

**Resources:**
- Memory: 128MB-256MB
- CPU: 0.25-0.5 cores
- Storage: `redis_data` volume

**Ports:**
- 6379 (Redis protocol)

---

## 🎯 Benefits of Simplified Setup

### 1. Fewer Resources
- **Before**: 2 app containers + 1 cache = 3 containers
- **After**: 1 app container + 1 cache = 2 containers
- **Savings**: ~512MB memory, 1 CPU core

### 2. Simpler Configuration
- Single port to remember (3000)
- One container to monitor
- Fewer environment variables
- Easier networking

### 3. Faster Startup
- One app container to build/start
- Faster healthchecks
- Quicker deployments

### 4. Single Source of Truth
- Both API and Web UI share same database connection
- No sync issues
- Consistent behavior

---

## 🛠️ Common Tasks

### View Logs
```bash
# All logs
docker compose logs

# Follow live logs
docker compose logs -f rustassistant

# Auto-scanner logs
docker compose logs -f rustassistant | grep auto_scanner
```

### Restart Server
```bash
docker compose restart rustassistant
```

### Rebuild After Code Changes
```bash
docker compose build rustassistant
docker compose up -d
```

### Check Health
```bash
# Main health endpoint
curl http://localhost:3000/health

# Web UI
curl http://localhost:3000/

# API
curl http://localhost:3000/api/repos

# Redis
docker compose exec redis redis-cli ping
```

### Update Configuration
```bash
# Edit .env file
nano .env

# Add/change variables
AUTO_SCAN_INTERVAL=30
RUST_LOG=debug

# Restart to apply
docker compose restart rustassistant
```

---

## 🔧 Troubleshooting

### Port 3000 Already in Use
```bash
# Option 1: Stop conflicting service
lsof -ti:3000 | xargs kill -9

# Option 2: Change port in .env
echo "PORT=3002" >> .env
docker compose up -d
```

### Container Won't Start
```bash
# Check logs
docker compose logs rustassistant

# Common issues:
# - Database locked: rm data/rustassistant.db-wal
# - Port conflict: Change PORT in .env
# - Redis not ready: docker compose up -d redis && sleep 5 && docker compose up -d rustassistant
```

### "Connection Refused" Errors
```bash
# Wait for healthcheck
docker compose ps

# Should show "Up (healthy)" after ~40 seconds
# If stuck in "Up (health: starting)", check logs
```

### Old API Scripts Not Working
```bash
# Update base URL in your scripts
# OLD: BASE_URL="http://localhost:3000"
# NEW: BASE_URL="http://localhost:3000"

# Example fix:
sed -i 's|localhost:3000|localhost:3000|g' your-script.sh
```

---

## 📊 Resource Comparison

### Before (3 Containers)
```
rustassistant-api:   512MB RAM, 1 CPU
rustassistant-web:   1GB RAM,   2 CPU
rustassistant-redis: 256MB RAM, 0.5 CPU
Total:               1.75GB RAM, 3.5 CPU
```

### After (2 Containers)
```
rustassistant:       1GB RAM,   2 CPU
rustassistant-redis: 256MB RAM, 0.5 CPU
Total:               1.25GB RAM, 2.5 CPU
```

**Savings: 500MB RAM, 1 CPU core**

---

## 🚀 Production Deployment

### Using docker-compose.prod.yml

```bash
# Pull pre-built image from Docker Hub
docker compose -f docker-compose.prod.yml pull

# Start services
docker compose -f docker-compose.prod.yml up -d

# Check status
docker compose -f docker-compose.prod.yml ps
```

### Environment Variables for Production

Create `.env.production`:
```bash
# Server
PORT=3000
HOST=0.0.0.0
RUST_LOG=info,rustassistant=info

# Database
DATABASE_URL=sqlite:/app/data/rustassistant.db

# Auto-Scanner
AUTO_SCAN_ENABLED=true
AUTO_SCAN_INTERVAL=60
AUTO_SCAN_MAX_CONCURRENT=2

# API Keys
XAI_API_KEY=your-key-here
XAI_BASE_URL=https://api.x.ai/v1

# Redis
REDIS_URL=redis://redis:6379

# Repository Path (customize for your system)
REPOS_PATH=/home/jordan/github
```

Load and start:
```bash
docker compose -f docker-compose.prod.yml --env-file .env.production up -d
```

---

## 📋 Docker Compose Overview

### docker-compose.yml (Development)
```yaml
services:
  rustassistant:
    build: .
    ports: ["3000:3000"]
    volumes:
      - ./data:/app/data
      - /home/jordan/github:/home/jordan/github:ro
  
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

### docker-compose.prod.yml (Production)
```yaml
services:
  rustassistant:
    image: nuniesmith/rustassistant:latest  # Pre-built
    ports: ["3000:3000"]
    volumes:
      - ./data:/app/data
      - ${REPOS_PATH}:${REPOS_PATH}:ro
  
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

---

## ✅ Verification Steps

After migration, verify everything works:

```bash
# 1. Check containers are running
docker compose ps
# Expected: 2 containers, both healthy

# 2. Test health endpoint
curl http://localhost:3000/health
# Expected: {"status":"ok","service":"rustassistant","version":"..."}

# 3. Test Web UI
curl -I http://localhost:3000/
# Expected: HTTP/1.1 200 OK

# 4. Test API
curl http://localhost:3000/api/repos
# Expected: JSON array of repositories

# 5. Check auto-scanner
docker compose logs rustassistant | grep -i "auto.*scan"
# Expected: "Starting auto-scanner" or similar

# 6. Open in browser
open http://localhost:3000
# Expected: Dashboard loads with stats
```

---

## 🎯 What Didn't Change

- Database schema (100% compatible)
- API endpoints (same paths, just different port)
- Web UI features (all working)
- Auto-scanner functionality
- Redis caching
- Data persistence
- Configuration options

---

## 📚 Additional Resources

- [WEB_UI_QUICKSTART.md](./WEB_UI_QUICKSTART.md) - Web UI usage guide
- [WEB_UI_STATUS.md](./WEB_UI_STATUS.md) - Full feature documentation
- [AUTO_SCANNER_SETUP.md](./AUTO_SCANNER_SETUP.md) - Auto-scanner configuration
- [docker/README.md](./docker/README.md) - Docker infrastructure

---

## 🆘 Need Help?

### Check Logs
```bash
docker compose logs -f rustassistant
```

### Database Inspection
```bash
sqlite3 data/rustassistant.db "SELECT * FROM repositories;"
```

### Container Shell Access
```bash
docker compose exec rustassistant /bin/bash
```

### Full Reset (Nuclear Option)
```bash
docker compose down -v
rm -rf data/rustassistant.db*
docker compose up -d
```

---

**Status**: ✅ Production Ready  
**Migration Effort**: < 5 minutes  
**Downtime**: < 1 minute  
**Data Loss**: None (database compatible)  
**Last Updated**: 2024-01-15