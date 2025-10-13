# syntax=docker/dockerfile:1
# Use llama.cpp MUSA base image as runtime foundation
FROM ghcr.io/bodhisearch/llama.cpp:latest-musa AS runtime

# Use pre-built BodhiApp binary from app-binary base image
ARG APP_BINARY_IMAGE
FROM ${APP_BINARY_IMAGE} AS app-binary

# === FINAL STAGE ===
FROM runtime

# Switch to root for file operations
USER root

# Copy BodhiApp binary from app-binary image and set ownership
COPY --from=app-binary /app/bodhi /app/bodhi
RUN chown llama:llama /app/bodhi && chmod +x /app/bodhi

# Configure BodhiApp environment (only keep RUST_LOG as it's not managed by SettingService)
ENV RUST_LOG=info

# Re-declare build arguments for final stage
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA

# Create defaults.yaml with MUSA-optimized configuration
COPY <<EOF /app/defaults.yaml
# BodhiApp Default Configuration for MUSA
# System paths and directories
BODHI_HOME: /data/bodhi_home
HF_HOME: /data/hf_home

# Version information
BODHI_VERSION: ${BODHI_VERSION}
BODHI_COMMIT_SHA: ${BODHI_COMMIT_SHA}

# Server configuration
BODHI_HOST: "0.0.0.0"
BODHI_PORT: 8080

# Build configuration - MUSA variant
BODHI_EXEC_LOOKUP_PATH: /app/bin
BODHI_EXEC_TARGET: x86_64-unknown-linux-gnu
BODHI_EXEC_VARIANTS: musa,cpu
BODHI_EXEC_VARIANT: musa
BODHI_EXEC_NAME: llama-server

# Server arguments
BODHI_LLAMACPP_ARGS: "--jinja --no-webui"
BODHI_LLAMACPP_ARGS_MUSA: "--n-gpu-layers -1 --threads 8 --batch-size 2048 --ubatch-size 512 --no-mmap --mlock"
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

# Health check (base image includes MUSA availability check)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
