# Use llama.cpp CPU base image as runtime foundation
FROM ghcr.io/bodhisearch/llama.cpp:latest-cpu AS runtime

# Build BodhiApp binary (CPU-only Rust build)
FROM rust:1.87.0-bookworm AS builder

# Build arguments for platform info and build variant
ARG BUILD_VARIANT=production
ARG CI_DOCKER=true
ENV BUILD_VARIANT=${BUILD_VARIANT}
ENV CI_DOCKER=${CI_DOCKER}

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

# Build bodhi binary with consistent optimization level
# Note: llama_server_proc will skip with CI_DOCKER=true
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
ENV BODHI_EXEC_LOOKUP_PATH=/app
ENV BODHI_HOST="0.0.0.0"
ENV BODHI_PORT="8080"

# Create data directories with proper ownership
RUN mkdir -p /data/bodhi_home /data/hf_home && \
    chown -R llama:llama /data

# Switch back to non-root user
USER llama
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
