# RustAssistant Docker Documentation

This directory contains Docker configuration for building and deploying RustAssistant API and Web UI services.

## 📋 Table of Contents

- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Building Images](#building-images)
- [Running Services](#running-services)
- [Environment Variables](#environment-variables)
- [Multi-Architecture Support](#multi-architecture-support)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

## 🏗️ Architecture

The RustAssistant Docker setup consists of three services:

1. **API Service** (Port 3000) - REST API server
2. **Web UI Service** (Port 3000) - Web interface
3. **Redis Service** (Port 6379) - LLM response caching

### Single Dockerfile, Multiple Services

We use a **single multi-stage Dockerfile** with build arguments to create both API and Web images:

```bash
# Build API variant
docker build --build-arg SERVICE_TYPE=api --build-arg SERVICE_PORT=3000 -t rustassistant:api .

# Build Web variant
docker build --build-arg SERVICE_TYPE=web --build-arg SERVICE_PORT=3000 -t rustassistant:web .
```

## 🚀 Quick Start

### Development Environment

```bash
# Start all services (API, Web UI, Redis)
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

Services will be available at:
- API: http://localhost:3000
- Web UI: http://localhost:3000
- Redis: localhost:6379

### Production Environment

```bash
# Start production services
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose -f docker-compose.prod.yml logs -f

# Stop services
docker compose -f docker-compose.prod.yml down
```

## 🔨 Building Images

### Using the Build Script (Recommended)

```bash
# Build both API and Web images
./scripts/build-docker.sh

# Build only API
./scripts/build-docker.sh --service api

# Build only Web UI
./scripts/build-docker.sh --service web

# Build and push to Docker Hub
./scripts/build-docker.sh --push --tag v1.0.0

# Build for specific platform (e.g., ARM64 for Raspberry Pi)
./scripts/build-docker.sh --platform linux/arm64
```

### Manual Building

```bash
# Build API image
docker build \
  -f docker/Dockerfile \
  --build-arg SERVICE_TYPE=api \
  --build-arg SERVICE_PORT=3000 \
  -t nuniesmith/rustassistant:api .

# Build Web UI image
docker build \
  -f docker/Dockerfile \
  --build-arg SERVICE_TYPE=web \
  --build-arg SERVICE_PORT=3000 \
  -t nuniesmith/rustassistant:web .
```

### Using Docker Compose

```bash
# Build all services defined in docker-compose.yml
docker compose build

# Build specific service
docker compose build api
docker compose build rustassistant-web

# Build with no cache
docker compose build --no-cache
```

## ▶️ Running Services

### Development Mode

```bash
# Start all services in background
docker compose up -d

# Start with live logs
docker compose up

# Start specific service
docker compose up api
docker compose up rustassistant-web

# Restart a service
docker compose restart api

# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v
```

### Production Mode

```bash
# Start production stack
docker compose -f docker-compose.prod.yml up -d

# Scale services (if needed)
docker compose -f docker-compose.prod.yml up -d --scale rustassistant-web=2

# View service status
docker compose -f docker-compose.prod.yml ps

# View resource usage
docker stats rustassistant-api rustassistant-web rustassistant-redis
```

## 🔧 Environment Variables

### Required Variables

Create a `.env` file in the project root:

```env
# X.AI API Configuration
XAI_API_KEY=your-xai-api-key-here
XAI_BASE_URL=https://api.x.ai/v1

# Optional: Custom port for API
PORT=3000

# Optional: Logging level
RUST_LOG=info,rustassistant=debug
```

### Service-Specific Variables

#### API Service (Port 3000)
```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite:/app/data/rustassistant.db
REDIS_URL=redis://redis:6379
RUST_LOG=info,rustassistant=debug
```

#### Web UI Service (Port 3000)
```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite:/app/data/rustassistant.db
CACHE_DB_PATH=/app/data/rustassistant_cache.db
REDIS_URL=redis://redis:6379
RUST_LOG=info,rustassistant=info
```

#### Redis Service
```env
# Redis configuration is handled via command args in docker-compose.yml
# Max memory: 256MB
# Eviction policy: allkeys-lru
# Persistence: AOF + RDB snapshots
```

### Environment Variable Override

You can override any environment variable at runtime:

```bash
# Override API port
PORT=3500 docker compose up api

# Override log level
RUST_LOG=debug docker compose up
```

## 🌍 Multi-Architecture Support

The Dockerfile supports both **AMD64** and **ARM64** architectures (including Raspberry Pi).

### Building Multi-Arch Images

```bash
# Enable Docker buildx
docker buildx create --use

# Build for multiple platforms
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg SERVICE_TYPE=web \
  --build-arg SERVICE_PORT=3000 \
  -t nuniesmith/rustassistant:latest \
  --push \
  .
```

### Platform-Specific Builds

```bash
# AMD64 only (most servers)
./scripts/build-docker.sh --platform linux/amd64

# ARM64 only (Raspberry Pi, M1/M2 Mac)
./scripts/build-docker.sh --platform linux/arm64

# Both platforms
./scripts/build-docker.sh --platform linux/amd64,linux/arm64
```

## 🚢 Production Deployment

### Raspberry Pi Deployment

```bash
# Pull pre-built ARM64 images
docker compose -f docker-compose.prod.yml pull

# Start services
docker compose -f docker-compose.prod.yml up -d

# Check health
docker compose -f docker-compose.prod.yml ps
```

### Server Deployment

```bash
# SSH into server
ssh user@your-server.com

# Clone repository
git clone https://github.com/nuniesmith/rustassistant.git
cd rustassistant

# Create .env file with secrets
nano .env

# Start production stack
docker compose -f docker-compose.prod.yml up -d

# Verify services are running
docker compose -f docker-compose.prod.yml ps
curl http://localhost:3000/health
curl http://localhost:3000/health
```

### Resource Limits

The services are configured with resource limits suitable for Raspberry Pi:

- **API**: 512MB RAM limit, 256MB reservation
- **Web UI**: 1GB RAM limit, 512MB reservation, 2 CPU limit
- **Redis**: 256MB RAM limit, 0.5 CPU limit

Adjust in `docker-compose.prod.yml` if needed.

## 🔍 Health Checks

All services include health checks:

```bash
# Check service health
docker compose ps

# View health check logs
docker inspect rustassistant-api --format='{{json .State.Health}}' | jq
docker inspect rustassistant-web --format='{{json .State.Health}}' | jq
docker inspect rustassistant-redis --format='{{json .State.Health}}' | jq

# Manual health check
curl http://localhost:3000/health
curl http://localhost:3000/health
docker exec rustassistant-redis redis-cli ping
```

## 📊 Monitoring & Logs

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f api
docker compose logs -f rustassistant-web
docker compose logs -f redis

# Last 100 lines
docker compose logs --tail=100

# Since specific time
docker compose logs --since 2024-01-01T10:00:00
```

### Monitor Resources

```bash
# Real-time stats
docker stats

# Specific services
docker stats rustassistant-api rustassistant-web rustassistant-redis
```

## 🐛 Troubleshooting

### Service Won't Start

```bash
# Check logs
docker compose logs api

# Check if port is already in use
sudo lsof -i :3000
sudo lsof -i :3000

# Restart service
docker compose restart api

# Rebuild and restart
docker compose up -d --build api
```

### Database Issues

```bash
# Check database file exists
ls -lh data/rustassistant.db

# Run database migrations
docker compose exec api rustassistant-server migrate

# Reset database (WARNING: deletes data)
rm data/rustassistant.db
docker compose restart api
```

### Redis Connection Issues

```bash
# Check Redis is running
docker compose ps redis

# Test Redis connection
docker compose exec redis redis-cli ping

# Check Redis logs
docker compose logs redis

# Restart Redis
docker compose restart redis
```

### Permission Issues

```bash
# Fix data directory permissions
sudo chown -R 1000:1000 data/

# Verify permissions
ls -la data/
```

### Out of Memory

```bash
# Check memory usage
docker stats

# Increase memory limits in docker-compose.yml
# Then restart:
docker compose down
docker compose up -d
```

### Clean Restart

```bash
# Stop all services
docker compose down

# Remove volumes (WARNING: deletes data)
docker compose down -v

# Rebuild images
docker compose build --no-cache

# Start fresh
docker compose up -d
```

## 📦 Data Persistence

Data is persisted in the following locations:

- **SQLite databases**: `./data/` (mounted volume)
- **Redis data**: `rustassistant_redis_data` (named volume)
- **Config files**: `./config/` (read-only mount)

### Backup Data

```bash
# Backup SQLite databases
cp -r data/ backup-$(date +%Y%m%d)/

# Backup Redis data
docker compose exec redis redis-cli BGSAVE
docker cp rustassistant-redis:/data/dump.rdb backup-redis-$(date +%Y%m%d).rdb
```

### Restore Data

```bash
# Restore SQLite databases
cp -r backup-20240101/ data/

# Restore Redis
docker cp backup-redis-20240101.rdb rustassistant-redis:/data/dump.rdb
docker compose restart redis
```

## 🔐 Security Best Practices

1. **Never commit `.env` file** - Contains sensitive API keys
2. **Use non-root user** - Dockerfile already configures this
3. **Read-only mounts** - Config directory is mounted read-only
4. **Network isolation** - Services use dedicated bridge network
5. **Resource limits** - All services have memory/CPU limits
6. **Health checks** - Automatic service monitoring
7. **Regular updates** - Keep base images updated

## 📚 Additional Resources

- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Dockerfile Best Practices](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
- [RustAssistant GitHub](https://github.com/nuniesmith/rustassistant)

## 📝 License

See the main project LICENSE file.