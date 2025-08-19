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

# Install system dependencies for building (removed cmake and build-essential since we're downloading binaries)
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (LTS version 22)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs

# Set working directory
WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crate source files
COPY crates/ crates/
COPY xtask/ xtask/

# Set CI environment variables to download pre-built binaries
ENV CI=true
ENV CI_RELEASE=true

# First build llama_server_proc to download pre-built llama-server binaries from GitHub releases
RUN cargo build --release -p llama_server_proc --locked

# Then build bodhi binary without native feature (server mode only)
# Use BUILD_VARIANT to determine features: "production" or "development" (default: production)
RUN if [ "$BUILD_VARIANT" = "development" ]; then \
      cargo build --release --bin bodhi --no-default-features --locked; \
    else \
      cargo build --release --bin bodhi --no-default-features --locked --features production; \
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
COPY --from=builder /build/target/release/bodhi /app/bodhi
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

# Create volumes for persistent data
VOLUME ["/data/bodhi_home", "/data/hf_home"]

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
