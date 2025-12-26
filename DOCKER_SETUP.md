# Docker Setup Guide - Speed Chess Betting

> Complete guide for judges and developers to run the application using Docker

---

## Table of Contents

1. [Quick Start (TL;DR)](#quick-start-tldr)
2. [Prerequisites](#prerequisites)
3. [Architecture Overview](#architecture-overview)
4. [Detailed Setup Instructions](#detailed-setup-instructions)
5. [Service Access Points](#service-access-points)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Configuration](#advanced-configuration)
8. [Build Performance](#build-performance)
9. [Cleanup & Reset](#cleanup--reset)

---

## Quick Start (TL;DR)

### For Judges - One Command Deployment

```bash
# Clone repository
git clone <repository-url>
cd speed-chess-betting

# Start everything
docker compose up

# Wait 2-3 minutes for initialization, then access:
# - Frontend: http://localhost:5173
# - GraphQL: http://localhost:8081
# - Faucet: http://localhost:8080
```

**Expected Timeline:**
- Build: 15-20 minutes (first time only)
- Network initialization: 30-60 seconds
- Contract deployment: 10-20 seconds
- Services ready: ~2 minutes total

---

## Prerequisites

### System Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| **RAM** | 4GB available | 8GB available |
| **Disk Space** | 10GB free | 15GB free |
| **CPU** | 2 cores | 4+ cores |
| **Docker Version** | 20.10+ | Latest stable |
| **Docker Compose** | v2.0+ | v2.20+ |

### Software Dependencies

1. **Docker Desktop** (Windows/Mac) or **Docker Engine** (Linux)
   - Download: https://docs.docker.com/get-docker/
   - Verify: `docker --version`

2. **Docker Compose V2**
   - Bundled with Docker Desktop
   - Linux: `sudo apt-get install docker-compose-plugin`
   - Verify: `docker compose version`

3. **Available Ports**
   ```bash
   # Ensure these ports are free
   5173  # Frontend
   8080  # Faucet
   8081  # GraphQL
   9000  # Validator 0
   9001  # Validator 1
   9002  # Validator 2
   ```

### Check Port Availability

```bash
# Linux/Mac
netstat -tuln | grep -E '5173|8080|8081|9000|9001|9002'

# Windows (PowerShell)
netstat -an | findstr "5173 8080 8081 9000 9001 9002"

# If ports are in use, stop conflicting services
```

---

## Architecture Overview

### Multi-Stage Docker Build

```
┌─────────────────────────────────────────────────────────────┐
│                    Dockerfile (Multi-Stage)                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Stage 1: contract-builder (rust:1.86-slim)                │
│    └── Build WASM contracts                                 │
│        ├── Install Rust dependencies                        │
│        ├── Add wasm32 target                                │
│        └── Compile contracts → target/*.wasm                │
│                                                              │
│  Stage 2: frontend-builder (node:20-slim)                  │
│    └── Build React frontend                                 │
│        ├── Install npm dependencies                         │
│        ├── Run TypeScript compilation                       │
│        └── Create production build → dist/                  │
│                                                              │
│  Stage 3: linera-runtime (ubuntu:22.04)                    │
│    └── Final runtime image                                  │
│        ├── Install Rust toolchain                           │
│        ├── Install Linera CLI (0.15.6)                      │
│        ├── Install Node.js 20                               │
│        ├── Copy WASM contracts from Stage 1                 │
│        ├── Copy frontend build from Stage 2                 │
│        └── Configure entrypoint script                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Service Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Docker Compose Services                    │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Service 1: linera-network                                   │
│    ├── Runs: Linera validators + GraphQL API                 │
│    ├── Ports: 8080 (Faucet), 8081 (GraphQL), 9000-9002      │
│    ├── Volumes: linera-data (persistent blockchain)          │
│    └── Health Check: curl http://localhost:8081              │
│                                                               │
│  Service 2: frontend                                         │
│    ├── Runs: Vite dev server                                 │
│    ├── Ports: 5173 (HTTP)                                    │
│    ├── Depends On: linera-network (healthy)                  │
│    ├── Volumes: Hot-reload source code                       │
│    └── Health Check: curl http://localhost:5173              │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

---

## Detailed Setup Instructions

### Step 1: Clone Repository

```bash
git clone <repository-url>
cd speed-chess-betting
```

### Step 2: Pre-flight Checks

```bash
# Check Docker installation
docker --version
# Expected: Docker version 20.10.0 or higher

# Check Docker Compose
docker compose version
# Expected: Docker Compose version v2.0.0 or higher

# Check available disk space
df -h .
# Expected: At least 10GB free

# Verify Docker daemon is running
docker ps
# Should not show connection errors
```

### Step 3: Build and Start Services

```bash
# Build images and start services
docker compose up

# Or run in detached mode (background)
docker compose up -d

# Watch logs in real-time
docker compose logs -f
```

### Step 4: Monitor Startup

You'll see output in this order:

1. **Building Images** (~15-20 minutes first time)
   ```
   [+] Building 1200.5s (45/45) FINISHED
   => [contract-builder 1/8] FROM rust:1.86-slim
   => [frontend-builder 1/6] FROM node:20-slim
   => [linera-runtime 1/15] FROM ubuntu:22.04
   ```

2. **Container Startup**
   ```
   [+] Running 2/2
   ✔ Container speed-chess-linera      Started
   ✔ Container speed-chess-frontend    Started
   ```

3. **Network Initialization** (~30 seconds)
   ```
   [INFO] Checking Rust installation...
   [SUCCESS] Rust version: rustc 1.86.0
   [INFO] Starting Linera local network...
   [SUCCESS] Linera network initialized successfully
   ```

4. **Contract Deployment** (~10-20 seconds)
   ```
   [INFO] Deploying contracts with wallet 0...
   [SUCCESS] Contracts deployed successfully
   ```

5. **Services Ready**
   ```
   [SUCCESS] GraphQL service is ready!
   [INFO] Starting Vite development server on port 5173...
   VITE v5.0.11  ready in 1234 ms
   ➜  Local:   http://localhost:5173/
   ```

### Step 5: Verify Services

```bash
# Check all services are healthy
docker compose ps

# Expected output:
NAME                    STATUS          PORTS
speed-chess-linera      Up (healthy)    0.0.0.0:8080-8081->8080-8081/tcp, 0.0.0.0:9000-9002->9000-9002/tcp
speed-chess-frontend    Up (healthy)    0.0.0.0:5173->5173/tcp

# Test endpoints
curl http://localhost:8081  # GraphQL API
curl http://localhost:5173  # Frontend
curl http://localhost:8080  # Faucet
```

---

## Service Access Points

### Frontend (React App)

- **URL:** http://localhost:5173
- **Features:**
  - Interactive chessboard
  - Game creation and joining
  - Live betting interface
  - Real-time move synchronization
  - Wallet connection

### GraphQL API

- **URL:** http://localhost:8081
- **Playground:** http://localhost:8081/graphql (if enabled)
- **Available Queries:**
  ```graphql
  query GetGame($gameId: String!) {
    game(id: $gameId) {
      id
      status
      currentFen
      whitePlayer
      blackPlayer
    }
  }

  query ListGames {
    games {
      id
      status
      moveCount
    }
  }
  ```

### Linera Faucet

- **URL:** http://localhost:8080
- **Purpose:** Request test tokens for wallets
- **Usage:** Enter chain ID and request tokens

### Validator Nodes

- **Validator 0:** http://localhost:9000
- **Validator 1:** http://localhost:9001
- **Validator 2:** http://localhost:9002

---

## Troubleshooting

### Build Issues

#### Problem: "failed to solve: process "/bin/sh -c cargo build" did not complete"

**Solution:**
```bash
# Increase Docker memory allocation
# Docker Desktop → Settings → Resources → Memory: 8GB

# Or build without cache
docker compose build --no-cache
```

#### Problem: "COPY failed: no source files were specified"

**Solution:**
```bash
# Ensure you're in the project root
pwd  # Should show: /path/to/speed-chess-betting

# Check .dockerignore isn't excluding necessary files
cat .dockerignore
```

#### Problem: Build takes longer than 30 minutes

**Solution:**
```bash
# This is normal on slower systems. To speed up:
# 1. Close other applications
# 2. Increase Docker CPU allocation
# 3. Use --parallel flag (if available)
```

### Runtime Issues

#### Problem: "Error: Cannot find module" in frontend

**Solution:**
```bash
# Rebuild frontend dependencies
docker compose exec frontend npm ci

# Or rebuild the entire service
docker compose up --build frontend
```

#### Problem: "Connection refused" to GraphQL

**Solution:**
```bash
# Check if service is running
docker compose ps linera-network

# View logs
docker compose logs linera-network

# If unhealthy, restart
docker compose restart linera-network
```

#### Problem: "Port 5173 already in use"

**Solution:**
```bash
# Find what's using the port
lsof -i :5173  # Mac/Linux
netstat -ano | findstr :5173  # Windows

# Kill the process or change the port in docker-compose.yml
ports:
  - "3000:5173"  # Map to different host port
```

### Network Issues

#### Problem: "Linera network initialization failed"

**Solution:**
```bash
# Remove existing blockchain data
docker compose down -v

# This deletes volumes and starts fresh
docker compose up
```

#### Problem: Containers can't communicate

**Solution:**
```bash
# Check network exists
docker network ls | grep linera

# Recreate network
docker compose down
docker compose up
```

### Performance Issues

#### Problem: Frontend hot reload is slow

**Solution:**
```bash
# This is expected in Docker. For development:
# 1. Run frontend natively:
cd frontend
npm install
npm run dev

# 2. Keep only Linera network in Docker:
docker compose up linera-network
```

### Debugging Commands

```bash
# View all logs
docker compose logs

# View specific service logs
docker compose logs linera-network
docker compose logs frontend

# Follow logs in real-time
docker compose logs -f --tail=100

# Execute shell inside container
docker compose exec linera-network bash
docker compose exec frontend sh

# Inspect container
docker inspect speed-chess-linera

# Check resource usage
docker stats
```

---

## Advanced Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# .env
COMPOSE_PROJECT_NAME=speed-chess
RUST_LOG=debug
GRAPHQL_PORT=8081
FRONTEND_PORT=5173
```

### Custom Build Arguments

```yaml
# docker-compose.override.yml
version: '3.8'
services:
  linera-network:
    build:
      args:
        RUST_VERSION: 1.86
        LINERA_VERSION: 0.15.6
```

### Volume Management

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect speed-chess-betting_linera-data

# Backup blockchain data
docker run --rm -v speed-chess-betting_linera-data:/data -v $(pwd):/backup \
  ubuntu tar czf /backup/linera-backup.tar.gz /data

# Restore blockchain data
docker run --rm -v speed-chess-betting_linera-data:/data -v $(pwd):/backup \
  ubuntu tar xzf /backup/linera-backup.tar.gz -C /
```

### Development Mode

For faster iteration during development:

```yaml
# docker-compose.dev.yml
services:
  frontend:
    volumes:
      - ./frontend/src:/app/frontend/src  # Hot reload
    command: npm run dev -- --host 0.0.0.0
```

Usage:
```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up
```

---

## Build Performance

### First Build (Clean)

- **Contract Build:** 8-10 minutes
- **Frontend Build:** 2-3 minutes
- **Linera Install:** 5-7 minutes
- **Total:** ~15-20 minutes

### Subsequent Builds (Cached)

- **With Code Changes:** 2-5 minutes
- **Without Changes:** 10-30 seconds

### Optimization Tips

1. **Use Docker BuildKit:**
   ```bash
   export DOCKER_BUILDKIT=1
   docker compose build
   ```

2. **Leverage Layer Caching:**
   - Don't modify `Cargo.toml` unnecessarily
   - Separate dependency installation from code copy

3. **Parallel Builds:**
   ```bash
   docker compose build --parallel
   ```

4. **Prune Build Cache:**
   ```bash
   docker builder prune
   ```

---

## Cleanup & Reset

### Stop Services

```bash
# Stop containers (keep data)
docker compose stop

# Stop and remove containers (keep data)
docker compose down

# Stop and remove everything including volumes
docker compose down -v
```

### Remove Images

```bash
# Remove project images
docker compose down --rmi all

# Remove all unused images
docker image prune -a
```

### Full Cleanup

```bash
# Nuclear option - removes everything
docker compose down -v --rmi all
docker system prune -a --volumes

# Then rebuild from scratch
docker compose up --build
```

### Disk Space Recovery

```bash
# Check Docker disk usage
docker system df

# Remove unused data
docker system prune -a

# Clean specific components
docker container prune
docker image prune
docker volume prune
docker network prune
```

---

## Production Deployment

### Security Considerations

1. **Never expose to internet without:**
   - Reverse proxy (nginx/traefik)
   - SSL/TLS certificates
   - Authentication layer
   - Rate limiting
   - Firewall rules

2. **Change default configurations:**
   - PRNG seed (use random value)
   - Ports (use non-standard ports)
   - Add secrets management

### Production docker-compose.yml

```yaml
version: '3.8'
services:
  linera-network:
    restart: always
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./certs:/etc/nginx/certs
    depends_on:
      - frontend
```

---

## Support & Contact

### Common Issues

- Check [GitHub Issues](repository-url/issues)
- Review [Linera Documentation](https://docs.linera.io)
- Join [Discord Community](discord-link)

### Reporting Bugs

Include:
1. `docker --version` output
2. `docker compose version` output
3. Full error logs: `docker compose logs > logs.txt`
4. Steps to reproduce

---

## Appendix: Manual Build Steps

If you prefer to build manually:

### Build Contracts

```bash
cd contracts
cargo build --release --target wasm32-unknown-unknown
```

### Build Frontend

```bash
cd frontend
npm install
npm run build
```

### Run Linera Network

```bash
linera net up --testing-prng-seed 37 --extra-wallets 2
linera project publish-and-create --with-wallet 0
linera service --port 8081 --with-wallet 0
```

### Run Frontend

```bash
cd frontend
npm run dev -- --host 0.0.0.0 --port 5173
```

---

**Last Updated:** 2025-12-26
**Docker Version:** 24.0+
**Linera SDK:** 0.15.6
**Maintainer:** Speed Chess Betting Team
