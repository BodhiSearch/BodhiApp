# Build stage - multi-platform aware
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

# Use BUILDPLATFORM only if it's set (for multi-platform builds)
FROM rust:1.87.0-bookworm as builder

# Build arguments for GitHub PAT, platform info, and build variant
ARG GH_PAT
ARG TARGETARCH
ARG BUILD_VARIANT=production
ENV GH_PAT=${GH_PAT}
ENV TARGETARCH=${TARGETARCH}
ENV BUILD_VARIANT=${BUILD_VARIANT}

# Enable Rust build optimizations
ENV CARGO_INCREMENTAL=1
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV RUST_LOG=info

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (LTS version 22) - needed for all builds
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

# Set CI environment variables to download pre-built binaries
ENV CI=true
ENV CI_RELEASE=true

# Build llama_server_proc with consistent optimization level
RUN if [ "$BUILD_VARIANT" = "production" ]; then \
      echo "Building llama_server_proc for production (release mode)..." && \
      cargo build --release -p llama_server_proc; \
    else \
      echo "Building llama_server_proc for development (debug mode)..." && \
      cargo build -p llama_server_proc; \
    fi

# Build bodhi binary with consistent optimization level
RUN if [ "$BUILD_VARIANT" = "production" ]; then \
      echo "Building bodhi binary for production (release mode)..." && \
      cargo build --release --bin bodhi --no-default-features --features production; \
    else \
      echo "Building bodhi binary for development (debug mode)..." && \
      cargo build --bin bodhi --no-default-features; \
    fi

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    libgomp1 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r bodhi && useradd -r -g bodhi -d /app -s /bin/bash bodhi

# Create application directory and data directories
RUN mkdir -p /app /app/bin /data/bodhi_home /data/hf_home && \
    chown -R bodhi:bodhi /app /data

# Copy the built binary from builder stage
ARG BUILD_VARIANT=production
COPY --from=builder /build/target/*/bodhi /app/bodhi
RUN chown bodhi:bodhi /app/bodhi && chmod +x /app/bodhi

# Copy llama-server executables from builder stage (copied by bodhi build.rs)
COPY --from=builder /build/crates/bodhi/src-tauri/bin/ /app/bin/
RUN chown -R bodhi:bodhi /app/bin && find /app/bin -type f -exec chmod +x {} \;

# Switch to non-root user
USER bodhi
WORKDIR /app

# Set environment variables
ENV RUST_LOG=info
ENV HF_HOME=/data/hf_home
ENV BODHI_HOME=/data/bodhi_home
ENV BODHI_EXEC_LOOKUP_PATH=/app/bin
ENV BODHI_HOST="0.0.0.0"

# Expose port
EXPOSE 8080

# Create single volume for persistent data, as railway only allows single volume
VOLUME ["/data"]

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
