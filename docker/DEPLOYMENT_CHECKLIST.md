# RustAssistant Docker Deployment Checklist

## 📋 Pre-Deployment Checklist

### 1. Environment Setup
- [ ] `.env` file created with all required variables
- [ ] `XAI_API_KEY` set in `.env`
- [ ] `XAI_BASE_URL` configured (default: https://api.x.ai/v1)
- [ ] Ports 3000, 3001, and 6379 are available
- [ ] Docker and Docker Compose installed
- [ ] Docker Buildx enabled (for multi-arch builds)

### 2. Data Directories
- [ ] `./data` directory exists
- [ ] `./config` directory exists
- [ ] Proper permissions set (UID 1000)
  ```bash
  mkdir -p data config
  sudo chown -R 1000:1000 data/
  ```

### 3. Build Verification
- [ ] Dockerfile syntax validated
- [ ] Build script is executable (`chmod +x scripts/build-docker.sh`)
- [ ] Docker Compose files validated
  ```bash
  docker compose config
  docker compose -f docker-compose.prod.yml config
  ```

## 🔨 Build Phase

### Development Build
- [ ] Build API service
  ```bash
  docker compose build api
  ```
- [ ] Build Web UI service
  ```bash
  docker compose build rustassistant-web
  ```
- [ ] Verify images created
  ```bash
  docker images | grep rustassistant
  ```

### Production Build
- [ ] Build both services with script
  ```bash
  ./scripts/build-docker.sh
  ```
- [ ] Tag with version
  ```bash
  ./scripts/build-docker.sh --tag v1.0.0
  ```
- [ ] Multi-architecture build (if deploying to ARM64/Raspberry Pi)
  ```bash
  ./scripts/build-docker.sh --platform linux/amd64,linux/arm64
  ```

## 🚀 Deployment Phase

### Development Deployment
- [ ] Start all services
  ```bash
  docker compose up -d
  ```
- [ ] Check service status
  ```bash
  docker compose ps
  ```
- [ ] Verify all services show "healthy"
- [ ] Check logs for errors
  ```bash
  docker compose logs
  ```

### Production Deployment
- [ ] Pull latest images (if using registry)
  ```bash
  docker compose -f docker-compose.prod.yml pull
  ```
- [ ] Start production services
  ```bash
  docker compose -f docker-compose.prod.yml up -d
  ```
- [ ] Verify service status
  ```bash
  docker compose -f docker-compose.prod.yml ps
  ```
- [ ] Check resource usage
  ```bash
  docker stats
  ```

## ✅ Health Check Verification

### API Service (Port 3000)
- [ ] Health endpoint responds
  ```bash
  curl http://localhost:3000/health
  # Expected: {"status":"ok"}
  ```
- [ ] Docker health check passing
  ```bash
  docker inspect rustassistant-api --format='{{.State.Health.Status}}'
  # Expected: healthy
  ```

### Web UI Service (Port 3001)
- [ ] Health endpoint responds
  ```bash
  curl http://localhost:3001/health
  # Expected: {"status":"ok"}
  ```
- [ ] Docker health check passing
  ```bash
  docker inspect rustassistant-web --format='{{.State.Health.Status}}'
  # Expected: healthy
  ```
- [ ] Web interface accessible in browser
  ```
  http://localhost:3001
  ```

### Redis Service (Port 6379)
- [ ] Redis responding
  ```bash
  docker compose exec redis redis-cli ping
  # Expected: PONG
  ```
- [ ] Docker health check passing
  ```bash
  docker inspect rustassistant-redis --format='{{.State.Health.Status}}'
  # Expected: healthy
  ```
- [ ] Check Redis memory usage
  ```bash
  docker compose exec redis redis-cli INFO memory
  ```

## 🔍 Functional Testing

### API Testing
- [ ] Test database connection
- [ ] Test Redis connection
- [ ] Test XAI API integration
- [ ] Create test note/task
- [ ] Verify data persists after restart

### Web UI Testing
- [ ] Access dashboard
- [ ] Test authentication (if enabled)
- [ ] Create/view/edit content
- [ ] Verify static assets load
- [ ] Test responsive design

### Integration Testing
- [ ] API and Web UI communicate
- [ ] Redis caching works
- [ ] Database shared between services
- [ ] File uploads work (if applicable)

## 📊 Monitoring Setup

### Logging
- [ ] Configure log aggregation
- [ ] Set up log rotation
  ```bash
  # Add to docker-compose.yml
  logging:
    driver: "json-file"
    options:
      max-size: "10m"
      max-file: "3"
  ```
- [ ] Verify logs are accessible
  ```bash
  docker compose logs -f
  ```

### Metrics
- [ ] Monitor CPU usage
- [ ] Monitor memory usage
- [ ] Monitor disk usage
- [ ] Set up alerts for resource limits

### Backup Configuration
- [ ] Database backup scheduled
  ```bash
  # Add to crontab
  0 2 * * * cp -r /path/to/data /path/to/backup-$(date +\%Y\%m\%d)
  ```
- [ ] Redis persistence verified
  ```bash
  docker compose exec redis redis-cli BGSAVE
  ```
- [ ] Backup restoration tested

## 🔐 Security Verification

### Container Security
- [ ] Services run as non-root user (UID 1000)
- [ ] Read-only mounts configured for config
- [ ] Network isolation enabled
- [ ] Resource limits set
- [ ] No sensitive data in environment variables (use `.env`)

### Access Control
- [ ] `.env` file not committed to git
- [ ] API endpoints secured (if auth required)
- [ ] Firewall rules configured
- [ ] HTTPS configured (if public-facing)
- [ ] Docker socket not exposed

### Secrets Management
- [ ] API keys stored securely
- [ ] Database credentials protected
- [ ] No hardcoded secrets in code
- [ ] Environment variables validated

## 🌐 Network Configuration

### Internal Network
- [ ] Bridge network created (`rustassistant-network`)
- [ ] Services can communicate internally
- [ ] DNS resolution works between services
  ```bash
  docker compose exec api ping redis
  ```

### External Access
- [ ] Ports exposed correctly (3000, 3001, 6379)
- [ ] Reverse proxy configured (if using)
- [ ] SSL/TLS certificates installed (if public)
- [ ] Domain names configured (if applicable)

## 📦 Data Persistence

### Volume Verification
- [ ] Named volumes created
  ```bash
  docker volume ls | grep rustassistant
  ```
- [ ] Data persists after container restart
  ```bash
  docker compose restart
  # Verify data still exists
  ```
- [ ] Volume backups configured

### Database
- [ ] SQLite database initialized
- [ ] Migrations run successfully
- [ ] Data integrity verified
- [ ] Backup/restore tested

## 🔄 Rolling Updates

### Update Procedure
- [ ] Backup current state
- [ ] Pull new images
  ```bash
  docker compose pull
  ```
- [ ] Graceful shutdown
  ```bash
  docker compose down
  ```
- [ ] Start with new version
  ```bash
  docker compose up -d
  ```
- [ ] Verify health checks
- [ ] Rollback plan prepared

## 📱 Platform-Specific Checks

### AMD64 (Standard Servers)
- [ ] Images built for linux/amd64
- [ ] Performance optimized
- [ ] Memory limits appropriate

### ARM64 (Raspberry Pi)
- [ ] Images built for linux/arm64
- [ ] Resource limits adjusted for Pi
- [ ] Temperature monitoring enabled
- [ ] Storage optimization configured

## 🚨 Troubleshooting Readiness

### Common Issues Documented
- [ ] Port conflicts
- [ ] Permission issues
- [ ] Memory limits
- [ ] Database corruption
- [ ] Redis connection failures

### Support Tools Available
- [ ] Debug mode documented
- [ ] Log access configured
- [ ] Container shell access tested
  ```bash
  docker compose exec api bash
  ```

## 📚 Documentation

### User Documentation
- [ ] `docker/README.md` reviewed
- [ ] `docker/QUICKREF.md` available
- [ ] `DOCKER_REFACTOR_SUMMARY.md` read
- [ ] Team trained on deployment process

### Operational Documentation
- [ ] Runbooks created
- [ ] Incident response plan documented
- [ ] Escalation procedures defined
- [ ] Contact information updated

## ✨ Final Sign-Off

### Pre-Production
- [ ] All tests passing
- [ ] Security scan completed
- [ ] Performance baseline established
- [ ] Monitoring configured
- [ ] Backup/restore verified

### Production Launch
- [ ] Stakeholders notified
- [ ] Deployment window scheduled
- [ ] Rollback plan ready
- [ ] On-call rotation scheduled
- [ ] Post-deployment verification planned

### Post-Deployment
- [ ] Health checks green for 24 hours
- [ ] No critical errors in logs
- [ ] Performance metrics normal
- [ ] User feedback collected
- [ ] Lessons learned documented

---

## 📝 Notes

**Date Deployed:** _________________

**Deployed By:** _________________

**Version:** _________________

**Environment:** [ ] Dev [ ] Staging [ ] Production

**Special Considerations:**
- _________________________________________________
- _________________________________________________
- _________________________________________________

**Issues Encountered:**
- _________________________________________________
- _________________________________________________
- _________________________________________________

**Resolution:**
- _________________________________________________
- _________________________________________________
- _________________________________________________

---

## 🎯 Quick Command Reference

```bash
# Health check all services
curl http://localhost:3000/health && \
curl http://localhost:3001/health && \
docker compose exec redis redis-cli ping

# View all logs
docker compose logs -f

# Check resource usage
docker stats

# Restart all services
docker compose restart

# Full rebuild
docker compose down && \
docker compose build --no-cache && \
docker compose up -d
```

---

**Checklist Version:** 1.0  
**Last Updated:** 2024-01-01  
**Maintainer:** DevOps Team