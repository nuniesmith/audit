# Docker Deployment Guide

**RustAssistant** - Containerized deployment with Redis caching

---

## 🚀 Quick Start

### 1. Setup Environment Variables

Create a `.env` file in the project root:

```bash
# Copy the example
cp .env.example .env

# Edit with your API key
nano .env
```

Required variables:
```env
# Grok API Configuration (REQUIRED)
XAI_API_KEY=xai-your-api-key-here
XAI_BASE_URL=https://api.x.ai/v1

# Optional: PostgreSQL (for multi-user future)
# POSTGRES_PASSWORD=secure-password-here
```

### 2. Start Services

```bash
# Start web UI with Redis cache
docker compose up -d

# Or with build
docker compose up -d --build

# Check status
docker compose ps

# View logs
docker compose logs -f rustassistant-web
```

### 3. Access the Web UI

Open your browser to: **http://localhost:3000**

---

## 📦 Services Overview

### RustAssistant Web UI (`rustassistant-web`)
- **Port**: 3000
- **Purpose**: Main web interface for managing notes, repos, costs, and analysis
- **Database**: SQLite (persistent in `./data` volume)
- **Cache**: Redis for LLM response caching

### Redis Cache (`rustassistant-redis`)
- **Port**: 6379 (exposed for debugging)
- **Purpose**: LLM response caching for 400x speedup
- **Memory**: 512MB limit (LRU eviction policy)
- **Persistence**: AOF + RDB snapshots every 60s

### RustAssistant CLI (`rustassistant-cli`)
- **Profile**: `cli` (on-demand only)
- **Purpose**: Batch jobs, scheduled tasks, maintenance
- **Usage**: `docker compose run --rm rustassistant-cli <command>`

---

## 🔧 Configuration

### Environment Variables

**Web UI:**
```env
HOST=0.0.0.0                          # Bind address
PORT=3000                             # Web UI port
RUST_LOG=info,rustassistant=debug    # Logging level
XAI_API_KEY=xai-xxx                  # Grok API key (required)
DATABASE_PATH=/app/data/rustassistant.db
CACHE_DB_PATH=/app/data/rustassistant_cache.db
REDIS_URL=redis://redis:6379         # Redis connection
```

**Redis:**
```bash
--maxmemory 512mb                    # Memory limit
--maxmemory-policy allkeys-lru       # Eviction policy
--save 60 1                          # Snapshot every 60s if 1+ writes
--appendonly yes                     # AOF persistence
```

### Volume Mounts

```yaml
volumes:
  - ./data:/app/data              # Database files (SQLite)
  - ./templates:/app/templates:ro # HTML templates (read-only)
  - ./config:/app/config:ro       # Config files (read-only)
  - ./repos:/app/repos            # Repository clones (CLI only)
```

---

## 🎯 Common Operations

### Start/Stop Services

```bash
# Start all services
docker compose up -d

# Start with logs
docker compose up

# Stop all services
docker compose down

# Stop and remove volumes (CAUTION: deletes data!)
docker compose down -v

# Restart a specific service
docker compose restart rustassistant-web
```

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f rustassistant-web
docker compose logs -f redis

# Last 100 lines
docker compose logs --tail=100 rustassistant-web
```

### Execute CLI Commands

```bash
# Run CLI commands
docker compose run --rm rustassistant-cli rustassistant --help

# Add repository
docker compose run --rm rustassistant-cli rustassistant repo add /app/repos/my-project

# Review code
docker compose run --rm rustassistant-cli rustassistant review files src/main.rs

# Batch analysis
docker compose run --rm rustassistant-cli rustassistant analyze batch src/**/*.rs

# Check cache stats
docker compose run --rm rustassistant-cli rustassistant cache stats
```

### Access Redis CLI

```bash
# Connect to Redis
docker exec -it rustassistant-redis redis-cli

# Check memory usage
docker exec rustassistant-redis redis-cli INFO memory

# Get cache stats
docker exec rustassistant-redis redis-cli INFO stats

# Monitor commands in real-time
docker exec rustassistant-redis redis-cli MONITOR
```

### Database Backup

```bash
# Backup SQLite databases
docker compose exec rustassistant-web sqlite3 /app/data/rustassistant.db ".backup /app/data/backup-$(date +%Y%m%d).db"

# Or from host
cp data/rustassistant.db data/backup-$(date +%Y%m%d).db
cp data/rustassistant_cache.db data/backup-cache-$(date +%Y%m%d).db
```

---

## 🍓 Raspberry Pi 4 Deployment

### Prerequisites

```bash
# Install Docker on Raspberry Pi OS
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo apt-get update
sudo apt-get install -y docker-compose-plugin

# Verify
docker --version
docker compose version
```

### Optimizations for Raspberry Pi 4

**1. Reduce Memory Limits**

Edit `docker-compose.yml`:
```yaml
redis:
  deploy:
    resources:
      limits:
        memory: 256M    # Reduced from 512M
      reservations:
        memory: 128M
```

**2. Build on Pi (ARM64)**

```bash
# Clone repository
git clone https://github.com/jordanistan/rustassistant.git
cd rustassistant

# Build on the Pi (will take ~10-15 minutes)
docker compose build

# Start services
docker compose up -d
```

**3. Use Pre-built ARM Images (Future)**

Once published to Docker Hub:
```yaml
rustassistant-web:
  image: jordanistan/rustassistant:latest-arm64
  # Remove build: section
```

**4. Resource Monitoring**

```bash
# Monitor resource usage
docker stats

# Check Pi temperature
vcgencmd measure_temp

# Free up disk space
docker system prune -a --volumes
```

### Performance Tips for Pi 4

- **Use SSD instead of SD card** - Much faster I/O
- **Enable swap** - At least 2GB for compilation
- **Overclock carefully** - Helps with build times
- **Monitor temperature** - Use heatsink/fan
- **Build overnight** - Initial build is slow

---

## 🔄 CI/CD Pipeline (Future)

### GitHub Actions for Pi 4

`.github/workflows/deploy-pi.yml`:
```yaml
name: Deploy to Raspberry Pi

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build ARM64 image
        run: |
          docker buildx build \
            --platform linux/arm64 \
            -f docker/Dockerfile.web \
            -t rustassistant:latest .
      
      - name: Deploy to Pi
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.PI_HOST }}
          username: ${{ secrets.PI_USER }}
          key: ${{ secrets.PI_SSH_KEY }}
          script: |
            cd ~/rustassistant
            git pull
            docker compose pull
            docker compose up -d --force-recreate
```

### Local Deployment Script

`scripts/deploy-to-pi.sh`:
```bash
#!/bin/bash
set -e

PI_HOST=${PI_HOST:-raspberry.local}
PI_USER=${PI_USER:-pi}

echo "🚀 Deploying RustAssistant to Raspberry Pi"

# Build locally (or skip if building on Pi)
# docker buildx build --platform linux/arm64 -t rustassistant:latest .

# Copy files to Pi
rsync -avz --exclude 'target' --exclude '.git' \
  ./ ${PI_USER}@${PI_HOST}:~/rustassistant/

# Deploy on Pi
ssh ${PI_USER}@${PI_HOST} << 'EOF'
  cd ~/rustassistant
  docker compose down
  docker compose build
  docker compose up -d
  docker compose logs --tail=50
EOF

echo "✅ Deployment complete!"
echo "🌐 Access at: http://${PI_HOST}:3000"
```

---

## 🐛 Troubleshooting

### Web UI Won't Start

```bash
# Check logs
docker compose logs rustassistant-web

# Common issues:
# 1. Port 3000 already in use
sudo lsof -i :3000
# Change port in docker-compose.yml or .env

# 2. Missing API key
docker compose exec rustassistant-web env | grep XAI
# Add to .env file

# 3. Permission issues
sudo chown -R $USER:$USER ./data
```

### Redis Connection Issues

```bash
# Check Redis is running
docker compose ps redis

# Test connection
docker exec rustassistant-redis redis-cli ping
# Should return: PONG

# Check logs
docker compose logs redis
```

### High Memory Usage

```bash
# Check memory
docker stats

# Reduce Redis memory
docker exec rustassistant-redis redis-cli CONFIG SET maxmemory 256mb

# Clear cache
docker exec rustassistant-redis redis-cli FLUSHALL
```

### Build Failures on Pi 4

```bash
# Increase swap space
sudo dfallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Build with less parallelism
docker compose build --build-arg CARGO_BUILD_JOBS=1

# Or build directly
cargo build --release -j 1
```

---

## 📊 Monitoring & Maintenance

### Health Checks

```bash
# Check all services health
docker compose ps

# Test web UI
curl http://localhost:3000/

# Check Redis
docker exec rustassistant-redis redis-cli PING
```

### Log Rotation

Add to `docker-compose.yml`:
```yaml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

### Scheduled Maintenance

Crontab for automated tasks:
```bash
# Backup daily at 2 AM
0 2 * * * cd ~/rustassistant && cp data/rustassistant.db data/backup-$(date +\%Y\%m\%d).db

# Prune Redis cache weekly
0 3 * * 0 docker exec rustassistant-redis redis-cli --eval /usr/local/bin/prune-old-keys.lua

# Clean Docker images monthly
0 4 1 * * docker system prune -af --volumes --filter "until=720h"
```

---

## 🔒 Security Best Practices

### Production Checklist

- [ ] Use strong passwords (especially for PostgreSQL when enabled)
- [ ] Don't expose Redis port 6379 publicly
- [ ] Use `.env` file with proper permissions (600)
- [ ] Run containers as non-root user (already configured)
- [ ] Enable firewall on host
- [ ] Use HTTPS reverse proxy (nginx/Traefik)
- [ ] Regular backups
- [ ] Keep Docker and images updated

### Reverse Proxy with Nginx (Recommended)

```nginx
server {
    listen 80;
    server_name rustassistant.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## 📚 Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/)
- [Redis Documentation](https://redis.io/docs/)
- [Raspberry Pi Docker Guide](https://docs.docker.com/engine/install/debian/)

---

## 💡 Tips

1. **First Run**: Build takes 10-15 min on Pi 4, ~5 min on desktop
2. **Cache Warming**: Run batch analysis after first start to build cache
3. **Memory**: 4GB Pi 4 is sufficient, 8GB recommended for large repos
4. **Storage**: SSD highly recommended over SD card
5. **Backups**: Daily automated backups of SQLite databases

---

## ✅ Checklist for Production

- [ ] API key configured in `.env`
- [ ] Data directory has correct permissions
- [ ] Redis memory limit appropriate for system
- [ ] Health checks passing
- [ ] Logs rotating properly
- [ ] Backup strategy in place
- [ ] Reverse proxy configured (if public)
- [ ] Firewall rules set
- [ ] Monitoring enabled
- [ ] Documentation updated

---

**Status**: Ready for local and Raspberry Pi 4 deployment!  
**Next**: Run `docker compose up -d` and access http://localhost:3000 🚀