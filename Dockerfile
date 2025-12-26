# ===================================================================
# PRODUCTION-READY MULTI-STAGE DOCKERFILE
# Speed Chess Betting - Linera Buildathon Wave 5
# ===================================================================
# Build time: ~15-20 minutes
# Final image size: ~2GB
# Services: Linera Network, GraphQL API, Frontend
# ===================================================================

# ===================================================================
# Stage 1: Build Rust Contracts (WASM)
# ===================================================================
FROM rust:1.86-slim AS contract-builder

# Set working directory
WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    clang \
    git \
    && rm -rf /var/lib/apt/lists/*

# Add WASM target for contract compilation
RUN rustup target add wasm32-unknown-unknown

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all contract source code
COPY contracts/ ./contracts/

# Build all contracts in release mode with optimizations
# - opt-level = 'z': Optimize for size
# - lto = true: Link-time optimization
# - codegen-units = 1: Better optimization at cost of compile time
# - strip = true: Remove debug symbols
RUN cargo build --release --target wasm32-unknown-unknown

# Verify WASM artifacts were created
RUN ls -lh target/wasm32-unknown-unknown/release/*.wasm && \
    echo "✅ Contract build successful"

# ===================================================================
# Stage 2: Build Frontend (React + Vite)
# ===================================================================
FROM node:20-slim AS frontend-builder

WORKDIR /build/frontend

# Install dependencies first (better layer caching)
COPY frontend/package*.json ./
RUN npm ci --silent

# Copy frontend source
COPY frontend/ ./

# Build production bundle
# Creates optimized static files in dist/
RUN npm run build && \
    echo "✅ Frontend build successful" && \
    ls -lh dist/

# ===================================================================
# Stage 3: Linera Runtime Environment
# ===================================================================
FROM ubuntu:22.04

# Prevent interactive prompts during installation
ENV DEBIAN_FRONTEND=noninteractive

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    ca-certificates \
    libssl3 \
    git \
    bash \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain (needed for linera CLI build)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.86.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

# Install protobuf compiler (required by Linera)
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install Linera CLI and services (version 0.15.8)
# Using cargo install ensures compatibility with the contracts
RUN cargo install --locked --version 0.15.8 linera-service && \
    cargo install --locked --version 0.15.8 linera-storage-service && \
    echo "✅ Linera services installed" && \
    linera --version

# Install Node.js 20 for frontend serving
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g pnpm && \
    node --version && \
    npm --version

# Copy compiled WASM contracts from builder stage
COPY --from=contract-builder /build/target/wasm32-unknown-unknown/release/*.wasm /app/contracts/

# Copy frontend build artifacts
COPY --from=frontend-builder /build/frontend/dist /app/frontend/dist
COPY --from=frontend-builder /build/frontend/package*.json /app/frontend/

# Copy frontend source for dev mode
COPY frontend/ /app/frontend/

# Copy contract source for deployment
COPY contracts/ /app/contracts/
COPY Cargo.toml Cargo.lock /app/

# Install frontend dependencies for dev server
WORKDIR /app/frontend
RUN npm ci --silent

# Set working directory
WORKDIR /app

# Create directory for Linera data
RUN mkdir -p /root/.config/linera

# Copy entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Expose ports
# 8080: Linera Faucet
# 8081: GraphQL API
# 5173: Frontend (Vite dev server)
# 9000-9010: Linera validator nodes
EXPOSE 8080 8081 5173 9000 9001 9002 9003 9004 9005 9006 9007 9008 9009 9010

# Environment variables
ENV RUST_LOG=info
ENV LINERA_WALLET=/root/.config/linera/wallet.json
ENV LINERA_STORAGE=rocksdb:/root/.config/linera/wallet.db

# Health check - verify GraphQL service is responding
HEALTHCHECK --interval=10s --timeout=5s --retries=30 \
    CMD curl -sf http://localhost:8081 || exit 1

# Default command
ENTRYPOINT ["/docker-entrypoint.sh"]
