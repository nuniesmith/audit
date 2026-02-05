# RustAssistant Docker Quick Reference

## 🚀 Quick Commands

### Start Services
```bash
# Development (local build)
docker compose up -d

# Production (pre-built images)
docker compose -f docker-compose.prod.yml up -d

# With logs
docker compose up
```

### Stop Services
```bash
docker compose down              # Stop all
docker compose down -v           # Stop + remove volumes (⚠️ deletes data)
docker compose stop api          # Stop specific service
```

### Build Images
```bash
# Using helper script (recommended)
./scripts/build-docker.sh                    # Build both API + Web
./scripts/build-docker.sh --service api      # API only
./scripts/build-docker.sh --service web      # Web only
./scripts/build-docker.sh --push --tag v1.0  # Build + push to registry

# Manual
docker compose build                         # Build all
docker compose build --no-cache api          # Rebuild from scratch
```

### View Logs
```bash
docker compose logs -f                # All services (follow)
docker compose logs -f api            # API only
docker compose logs --tail=100        # Last 100 lines
```

### Health & Status
```bash
docker compose ps                     # Service status
docker stats                          # Resource usage
curl http://localhost:3000/health     # API health
curl http://localhost:3001/health     # Web health
```

### Restart Services
```bash
docker compose restart api            # Restart API
docker compose restart                # Restart all
docker compose up -d --force-recreate # Force recreate containers
```

## 🔧 Service URLs

| Service | Port | URL |
|---------|------|-----|
| API     | 3000 | http://localhost:3000 |
| Web UI  | 3001 | http://localhost:3001 |
| Redis   | 6379 | redis://localhost:6379 |

## 🛠️ Maintenance

### Update Images
```bash
# Pull latest from registry
docker compose pull

# Rebuild from source
docker compose build --pull
```

### Clean Up
```bash
docker compose down -v                    # Remove containers + volumes
docker system prune -a                    # Clean all unused resources
docker volume prune                       # Remove unused volumes
```

### Backup & Restore
```bash
# Backup
cp -r data/ backup-$(date +%Y%m%d)/
docker compose exec redis redis-cli BGSAVE

# Restore
cp -r backup-20240101/ data/
docker compose restart
```

## 🐛 Troubleshooting

### Service won't start
```bash
docker compose logs api               # Check logs
docker compose restart api            # Restart
docker compose up -d --build api      # Rebuild + restart
```

### Port conflict
```bash
sudo lsof -i :3000                    # Check what's using port
PORT=3500 docker compose up api       # Use different port
```

### Database issues
```bash
ls -lh data/rustassistant.db          # Check DB exists
sudo chown -R 1000:1000 data/         # Fix permissions
```

### Redis issues
```bash
docker compose exec redis redis-cli ping  # Test connection
docker compose restart redis              # Restart Redis
docker compose logs redis                 # Check logs
```

### Out of memory
```bash
docker stats                          # Check memory usage
# Edit docker-compose.yml to increase limits
docker compose down && docker compose up -d
```

## 📋 Environment Variables

Required in `.env`:
```env
XAI_API_KEY=your-key-here
XAI_BASE_URL=https://api.x.ai/v1
```

Optional:
```env
PORT=3000
RUST_LOG=info,rustassistant=debug
```

## 🌍 Multi-Platform

```bash
# Build for Raspberry Pi (ARM64)
./scripts/build-docker.sh --platform linux/arm64

# Build for both AMD64 + ARM64
./scripts/build-docker.sh --platform linux/amd64,linux/arm64 --push
```

## 📊 Build Arguments

```bash
# Dockerfile accepts:
--build-arg SERVICE_TYPE=api|web      # Service variant
--build-arg SERVICE_PORT=3000|3001    # Port number
```

## 🔍 Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug docker compose up

# Interactive shell in container
docker compose exec api bash
docker compose exec rustassistant-web bash

# Run one-off command
docker compose run --rm api rustassistant --help
```

## 📦 Production Tips

✅ Use `docker-compose.prod.yml` for production  
✅ Set resource limits in compose file  
✅ Enable health checks  
✅ Use Redis for LLM caching  
✅ Mount volumes for data persistence  
✅ Set `restart: unless-stopped`  
✅ Run as non-root user (already configured)  

## 🔗 Links

- Full docs: `docker/README.md`
- Build script: `scripts/build-docker.sh`
- Dockerfile: `docker/Dockerfile`
