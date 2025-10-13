# syntax=docker/dockerfile:1
# BodhiApp Binary Builder
# Multi-platform base image containing pre-built bodhi binary
# Used by all GPU variant Dockerfiles to avoid redundant compilation

FROM rust:1.87.0-bookworm AS builder

# Build arguments for platform info and build variant
ARG BUILD_VARIANT=production
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA
ARG TARGETARCH
ARG CI_EXEC_NAME=llama-server

ENV BUILD_VARIANT=${BUILD_VARIANT}
ENV CI_EXEC_NAME=${CI_EXEC_NAME}
ENV TARGETARCH=${TARGETARCH}

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
COPY async-openai/ async-openai/

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
# Note: CI_BUILD_TARGET is automatically set based on TARGETARCH for each platform
RUN export CI_BUILD_TARGET=$(case "$TARGETARCH" in \
      "arm64") echo "aarch64-unknown-linux-gnu" ;; \
      "amd64"|*) echo "x86_64-unknown-linux-gnu" ;; \
    esac) && \
    echo "Building bodhi binary for $TARGETARCH (target: $CI_BUILD_TARGET)..." && \
    if [ "$BUILD_VARIANT" = "production" ]; then \
      echo "Building bodhi binary for production (release mode)..." && \
      cargo build --release --bin bodhi --no-default-features --features production; \
    else \
      echo "Building bodhi binary for development (debug mode)..." && \
      cargo build --bin bodhi --no-default-features; \
    fi

# === FINAL MINIMAL IMAGE ===
# Create minimal image with just the bodhi binary
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
RUN mkdir -p /app

# Copy bodhi binary from builder
COPY --from=builder /build/target/*/bodhi /app/bodhi
RUN chmod +x /app/bodhi

# Set working directory
WORKDIR /app

# This image is meant to be used as a base for COPY operations
# No entrypoint or CMD needed
