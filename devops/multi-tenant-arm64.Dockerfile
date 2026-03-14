# syntax=docker/dockerfile:1

ARG APP_BINARY_IMAGE

FROM ${APP_BINARY_IMAGE} AS app-binary

# Use llama.cpp CPU base image as runtime foundation
FROM ghcr.io/bodhisearch/llama.cpp:latest-cpu AS runtime

# === FINAL STAGE ===
FROM runtime

# Switch to root for file operations
USER root

# Copy BodhiApp binary from app-binary image and set ownership
COPY --from=app-binary /app/bodhi /app/bodhi
RUN chown llama:llama /app/bodhi && chmod +x /app/bodhi

# Configure BodhiApp environment
ENV RUST_LOG=info

# Re-declare build arguments for final stage
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA

# Create defaults.yaml with multi-tenant configuration
RUN cat > /app/defaults.yaml << EOF
# BodhiApp Default Configuration for Multi-Tenant (ARM64)
# System paths and directories
BODHI_HOME: /data/bodhi_home
HF_HOME: /data/hf_home

# Version information
BODHI_VERSION: ${BODHI_VERSION}
BODHI_COMMIT_SHA: ${BODHI_COMMIT_SHA}

# Deployment mode
BODHI_DEPLOYMENT: multi_tenant

# Server configuration
BODHI_HOST: "0.0.0.0"
BODHI_PORT: 1135

# Build configuration - CPU variant (no-op in multi-tenant, included for consistency)
BODHI_EXEC_LOOKUP_PATH: /app/bin
BODHI_EXEC_TARGET: aarch64-unknown-linux-gnu
BODHI_EXEC_VARIANTS: cpu
BODHI_EXEC_VARIANT: cpu
BODHI_EXEC_NAME: llama-server

# Server arguments (no-op in multi-tenant)
BODHI_LLAMACPP_ARGS: "--jinja --no-webui"
BODHI_LLAMACPP_ARGS_CPU: "--threads 4 --no-mmap"
EOF

# Create data directories with proper ownership
RUN mkdir -p /data/bodhi_home /data/hf_home
RUN chown -R llama:llama /data

# Switch back to non-root user
USER llama
WORKDIR /app

# Expose port
EXPOSE 1135

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:1135/ping || exit 1

# Set entrypoint
ENTRYPOINT ["/app/bodhi", "serve"]
