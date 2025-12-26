# Quick Start - Speed Chess Betting

> For judges: Get the app running in under 5 minutes

---

## Step 1: Prerequisites Check

```bash
# Verify Docker is installed
docker --version
# Expected: Docker version 20.10+

# Verify Docker Compose
docker compose version
# Expected: Docker Compose version v2.0+

# Check available ports
netstat -tuln | grep -E '5173|8080|8081'
# Should return nothing (ports are free)
```

---

## Step 2: Clone and Start

```bash
# Clone repository
git clone <repository-url>
cd speed-chess-betting

# Start all services
docker compose up
```

---

## Step 3: Wait for Startup

**Watch for these messages in the logs:**

```
✅ Contract build successful
✅ Frontend build successful
✅ Linera services installed
[SUCCESS] Linera network initialized successfully
[SUCCESS] Contracts deployed successfully
[SUCCESS] GraphQL service is ready!
VITE v5.0.11 ready in 1234 ms
➜  Local:   http://localhost:5173/
```

**Total wait time:** 2-3 minutes after image is built

---

## Step 4: Access the App

Open your browser:

| Service | URL | Purpose |
|---------|-----|---------|
| **Frontend** | http://localhost:5173 | Main application UI |
| **GraphQL** | http://localhost:8081 | API endpoint |
| **Faucet** | http://localhost:8080 | Get test tokens |

---

## Step 5: Test the Application

### Create a Game

1. Open http://localhost:5173
2. Click "Connect Wallet"
3. Click "Create Game"
4. Make moves on the chessboard

### Place a Bet

1. Navigate to "Active Games"
2. Select a game
3. Choose White/Black/Draw
4. Enter bet amount
5. Confirm transaction

---

## Common Issues

### "Port already in use"

```bash
# Find what's using the port
lsof -i :5173  # Mac/Linux
netstat -ano | findstr :5173  # Windows

# Kill the process or change port in docker-compose.yml
```

### "Docker build failed"

```bash
# Increase Docker memory to 8GB
# Docker Desktop → Settings → Resources → Memory

# Rebuild without cache
docker compose build --no-cache
```

### "Services not healthy"

```bash
# View detailed logs
docker compose logs

# Restart services
docker compose restart

# Full reset
docker compose down -v
docker compose up
```

---

## Stop and Cleanup

```bash
# Stop services (keep data)
docker compose stop

# Stop and remove containers (keep data)
docker compose down

# Complete cleanup (removes all data)
docker compose down -v
docker system prune -a
```

---

## Verification Checklist

- [ ] Docker version 20.10+ installed
- [ ] Docker Compose v2.0+ installed
- [ ] Ports 5173, 8080, 8081 are free
- [ ] At least 8GB RAM available
- [ ] At least 15GB disk space free
- [ ] `docker compose up` completes without errors
- [ ] Frontend accessible at http://localhost:5173
- [ ] GraphQL accessible at http://localhost:8081
- [ ] Faucet accessible at http://localhost:8080
- [ ] Can create a chess game
- [ ] Can make moves
- [ ] Can place bets

---

## Need Help?

- **Detailed documentation:** [DOCKER_SETUP.md](./DOCKER_SETUP.md)
- **Full README:** [README.md](./README.md)
- **View logs:** `docker compose logs -f`
- **Check container status:** `docker compose ps`

---

**Expected Timeline:**
- First build: 15-20 minutes
- Subsequent starts: 30 seconds
- Network initialization: 30 seconds
- Total time to ready: ~2-3 minutes (after build)
