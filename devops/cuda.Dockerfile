# syntax=docker/dockerfile:1
# Use llama.cpp CUDA base image as runtime foundation
FROM ghcr.io/bodhisearch/llama.cpp:latest-cuda AS runtime

# Build BodhiApp binary (CPU-only Rust build)
FROM rust:1.87.0-bookworm AS builder

# Build arguments for platform info and build variant
ARG BUILD_VARIANT=production
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA
ARG CI_BUILD_TARGET=x86_64-unknown-linux-gnu
ARG CI_DEFAULT_VARIANT=cuda
ARG CI_BUILD_VARIANTS=cuda,cpu
ARG CI_EXEC_NAME=llama-server
ENV BUILD_VARIANT=${BUILD_VARIANT}
ENV CI_BUILD_TARGET=${CI_BUILD_TARGET}
ENV CI_DEFAULT_VARIANT=${CI_DEFAULT_VARIANT}
ENV CI_BUILD_VARIANTS=${CI_BUILD_VARIANTS}
ENV CI_EXEC_NAME=${CI_EXEC_NAME}

# Enable Rust build optimizations
ENV CARGO_INCREMENTAL=1
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV RUST_LOG=info

# Install system dependencies for building (Node.js still needed for frontend)
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (LTS version 22) - needed for frontend build
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs

# Set working directory
WORKDIR /build

# === DEPENDENCY CACHING STAGE ===
# Copy workspace configuration, filter script, and minimal crates for dependency pre-compilation
COPY Cargo.toml Cargo.lock ./
COPY scripts/filter-cargo-toml.py ./scripts/
COPY crates/ci_optims/ crates/ci_optims/

# Create filtered Cargo.toml for dependency-only build and generate new lock file
RUN python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml && \
    mv Cargo.filtered.toml Cargo.toml && \
    cargo generate-lockfile

# Pre-compile all heavy dependencies with consistent optimization level
RUN if [ "$BUILD_VARIANT" = "production" ]; then \
      echo "Pre-compiling dependencies for production (release mode)..." && \
      cargo build --release -p ci_optims; \
    else \
      echo "Pre-compiling dependencies for development (debug mode)..." && \
      cargo build -p ci_optims; \
    fi

# === TS CLIENT BUILD STAGE ===
# Copy TS client source and generate types
COPY ts-client/ ts-client/
COPY openapi.json ./
RUN cd ts-client && npm ci && npm run build:openapi

# === APPLICATION BUILD STAGE ===
# Copy all crate source files and restore original Cargo.toml
COPY crates/ crates/
COPY xtask/ xtask/

# Restore original Cargo.toml and regenerate lock file for full workspace
COPY Cargo.toml ./
RUN cargo generate-lockfile

# Build bodhi binary with consistent optimization level
# Note: llama_server_proc will use CI_BUILD_TARGET configuration
RUN if [ "$BUILD_VARIANT" = "production" ]; then \
      echo "Building bodhi binary for production (release mode)..." && \
      cargo build --release --bin bodhi --no-default-features --features production; \
    else \
      echo "Building bodhi binary for development (debug mode)..." && \
      cargo build --bin bodhi --no-default-features; \
    fi

# === FINAL STAGE ===
FROM runtime

# Switch to root for file operations
USER root

# Copy BodhiApp binary from builder and set ownership
COPY --from=builder /build/target/*/bodhi /app/bodhi
RUN chown llama:llama /app/bodhi && chmod +x /app/bodhi

# Configure BodhiApp environment (only keep RUST_LOG as it's not managed by SettingService)
ENV RUST_LOG=info

# Re-declare build arguments for final stage
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA

# Create defaults.yaml with CUDA-optimized configuration
COPY <<EOF /app/defaults.yaml
# BodhiApp Default Configuration for CUDA
# System paths and directories
BODHI_HOME: /data/bodhi_home
HF_HOME: /data/hf_home

# Version information
BODHI_VERSION: ${BODHI_VERSION}
BODHI_COMMIT_SHA: ${BODHI_COMMIT_SHA}

# Server configuration
BODHI_HOST: "0.0.0.0"
BODHI_PORT: 8080

# Build configuration - CUDA variant
BODHI_EXEC_LOOKUP_PATH: /app/bin
BODHI_EXEC_TARGET: x86_64-unknown-linux-gnu
BODHI_EXEC_VARIANTS: cuda,cpu
BODHI_EXEC_VARIANT: cuda
BODHI_EXEC_NAME: llama-server

# Server arguments
BODHI_LLAMACPP_ARGS: "--jinja --no-webui"
BODHI_LLAMACPP_ARGS_CUDA: "--n-gpu-layers -1 --flash-attn --batch-size 2048 --ubatch-size 512 --cache-type-k q8_0 --cache-type-v q8_0 --threads 8 --threads-batch 8 --cont-batching --parallel 1 --ctx-size 8192 --no-mmap --mlock"
BODHI_LLAMACPP_ARGS_CPU: "--threads 4 --no-mmap"
EOF

# Create data directories with proper ownership
RUN mkdir -p /data/bodhi_home /data/hf_home

RUN chown -R llama:llama /data

# Switch back to non-root user
USER llama
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check (base image includes CUDA availability check)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
