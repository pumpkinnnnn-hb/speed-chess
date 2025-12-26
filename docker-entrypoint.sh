#!/bin/bash

# ===================================================================
# DOCKER ENTRYPOINT SCRIPT
# Speed Chess Betting - Linera Buildathon Wave 5
# ===================================================================
# Purpose: Initialize Linera network, deploy contracts, start services
# Usage: Automatically executed by Docker container
# ===================================================================

set -e  # Exit on any error
set -u  # Exit on undefined variable

# ===================================================================
# ANSI Color Codes for Beautiful Output
# ===================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ===================================================================
# Logging Functions
# ===================================================================
log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${MAGENTA}==================================================================${NC}"
    echo -e "${MAGENTA} $1${NC}"
    echo -e "${MAGENTA}==================================================================${NC}"
    echo ""
}

# ===================================================================
# Error Handler
# ===================================================================
error_exit() {
    log_error "$1"
    log_error "Deployment failed. Check logs above for details."
    exit 1
}

# ===================================================================
# Trap errors
# ===================================================================
trap 'error_exit "An error occurred on line $LINENO"' ERR

# ===================================================================
# STEP 1: System Validation
# ===================================================================
log_section "STEP 1: System Validation"

log_info "Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    error_exit "Rust is not installed"
fi
log_success "Rust version: $(rustc --version)"

log_info "Checking Linera CLI installation..."
if ! command -v linera &> /dev/null; then
    error_exit "Linera CLI is not installed"
fi
log_success "Linera version: $(linera --version)"

log_info "Checking Node.js installation..."
if ! command -v node &> /dev/null; then
    error_exit "Node.js is not installed"
fi
log_success "Node.js version: $(node --version)"

log_success "All system dependencies validated"

# ===================================================================
# STEP 2: Initialize Linera Network
# ===================================================================
log_section "STEP 2: Initialize Linera Local Network"

# Check if wallet already exists
if [ -f "$LINERA_WALLET" ]; then
    log_warning "Wallet already exists at $LINERA_WALLET"
    log_info "Cleaning up existing network..."
    rm -rf /root/.config/linera/*
    log_success "Cleanup complete"
fi

log_info "Starting Linera local network with 3 validator nodes..."
log_info "Using PRNG seed: 37 (deterministic for testing)"

# Start the local network
# - testing-prng-seed: Makes network initialization deterministic
# - extra-wallets: Creates 3 wallet chains for multi-player testing
linera net up --testing-prng-seed 37 --extra-wallets 2 || error_exit "Failed to start Linera network"

log_success "Linera network initialized successfully"

# Display wallet info
log_info "Wallet location: $LINERA_WALLET"
log_info "Storage location: $LINERA_STORAGE"

# ===================================================================
# STEP 3: Deploy Contracts
# ===================================================================
log_section "STEP 3: Deploy Smart Contracts"

cd /app/contracts || error_exit "Contracts directory not found"

log_info "Deploying contracts with wallet 0..."

# Deploy and publish contracts
# This creates the application on-chain and returns the Application ID
if linera project publish-and-create --with-wallet 0; then
    log_success "Contracts deployed successfully"
else
    error_exit "Contract deployment failed"
fi

# Extract application IDs from deployment output
# Note: In production, you'd parse the actual output
log_info "Contract deployment complete"

cd /app || exit 1

# ===================================================================
# STEP 4: Configure Environment
# ===================================================================
log_section "STEP 4: Configure Frontend Environment"

# Get the default chain ID
DEFAULT_CHAIN=$(linera wallet show | grep -oP 'Default chain:\s+\K[a-f0-9]+' | head -n 1)

if [ -z "$DEFAULT_CHAIN" ]; then
    log_warning "Could not extract default chain ID"
    DEFAULT_CHAIN="unknown"
else
    log_success "Default chain: $DEFAULT_CHAIN"
fi

# Create frontend environment file
cat > /app/frontend/.env << EOF
# Auto-generated by docker-entrypoint.sh
VITE_GRAPHQL_URL=http://localhost:8081
VITE_FAUCET_URL=http://localhost:8080
VITE_NETWORK_MODE=local
VITE_DEFAULT_CHAIN=$DEFAULT_CHAIN
EOF

log_success "Frontend environment configured"

# ===================================================================
# STEP 5: Start GraphQL Service (Background)
# ===================================================================
log_section "STEP 5: Start GraphQL Service"

log_info "Starting Linera GraphQL service on port 8081..."

# Start GraphQL service in background
nohup linera service --port 8081 --with-wallet 0 > /var/log/linera-graphql.log 2>&1 &
GRAPHQL_PID=$!

log_success "GraphQL service started with PID: $GRAPHQL_PID"

# Wait for GraphQL to be ready
log_info "Waiting for GraphQL service to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -sf http://localhost:8081 > /dev/null 2>&1; then
        log_success "GraphQL service is ready!"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo -n "."
    sleep 2
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    error_exit "GraphQL service failed to start within 60 seconds"
fi

echo ""  # New line after dots

# ===================================================================
# STEP 6: Start Frontend Service (Foreground)
# ===================================================================
log_section "STEP 6: Start Frontend Development Server"

cd /app/frontend || error_exit "Frontend directory not found"

log_info "Starting Vite development server on port 5173..."
log_info "Frontend will be accessible at http://localhost:5173"

# Start frontend in foreground (keeps container running)
exec npm run dev -- --host 0.0.0.0 --port 5173

# ===================================================================
# NOTE: The script never reaches here because exec replaces the process
# ===================================================================
