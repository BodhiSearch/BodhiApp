# syntax=docker/dockerfile:1
# Use llama.cpp ROCm base image as runtime foundation
FROM ghcr.io/bodhisearch/llama.cpp:latest-rocm AS runtime

# Build BodhiApp binary (CPU-only Rust build)
FROM rust:1.87.0-bookworm AS builder

# Build arguments for platform info and build variant
ARG BUILD_VARIANT=production
ARG CI_BUILD_TARGET=x86_64-unknown-linux-gnu
ARG CI_DEFAULT_VARIANT=rocm
ARG CI_BUILD_VARIANTS=rocm,cpu
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

# === APPLICATION BUILD STAGE ===
# Copy all crate source files and restore original Cargo.toml
COPY crates/ crates/
COPY xtask/ xtask/

# Restore original Cargo.toml and regenerate lock file for full workspace
COPY Cargo.toml ./
RUN cargo generate-lockfile

# === TS CLIENT BUILD STAGE ===
# Copy TS client source and build it
COPY ts-client/ ts-client/

# Build TS client (requires OpenAPI generation which needs the built Rust code)
WORKDIR /build/ts-client
RUN npm install && npm run build

# Return to build root
WORKDIR /build

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

# Configure BodhiApp environment
ENV RUST_LOG=info
ENV HF_HOME=/data/hf_home
ENV BODHI_HOME=/data/bodhi_home
ENV BODHI_EXEC_LOOKUP_PATH=/app/bin
ENV BODHI_HOST="0.0.0.0"
ENV BODHI_PORT="8080"

# Build Configuration
ENV CI_DEFAULT_VARIANT=rocm
ENV CI_BUILD_VARIANTS=rocm,cpu
ENV CI_EXEC_NAME=llama-server

# Server Arguments (visible and maintainable)
ENV BODHI_LLAMACPP_ARGS="--jinja --no-webui --keep 24"
ENV BODHI_LLAMACPP_ARGS_ROCM="--n-gpu-layers 999 --split-mode row --hipblas"
ENV BODHI_LLAMACPP_ARGS_CPU="--threads 4 --no-mmap"

# Create data directories with proper ownership
RUN mkdir -p /data/bodhi_home /data/hf_home

RUN chown -R llama:llama /data

# Switch back to non-root user
USER llama
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check (base image includes ROCm availability check)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
