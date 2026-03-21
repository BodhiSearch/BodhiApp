# Plan: Add Multi-Tenant Docker Variants (AMD64 + ARM64)

## Context

BodhiApp currently builds Docker images for standalone deployment only (CPU + 5 GPU variants). Multi-tenant mode (`BODHI_DEPLOYMENT=multi_tenant`) exists in the codebase but has no production Docker image. We need two multi-tenant Docker variants:

- **`multi-tenant`** — AMD64, for cloud/production deployment
- **`multi-tenant-arm64`** — ARM64, for macOS local development

Both bake `BODHI_DEPLOYMENT: multi_tenant` into `defaults.yaml` (immutable system setting), expect PostgreSQL URLs and dashboard credentials as runtime env vars, and build alongside existing variants in CI. They are **separate single-platform images** (not one multi-platform image) to avoid slow multi-platform builds.

**Future TECHDEBT**: Replace llama.cpp CPU base with `debian:bookworm-slim` for smaller image (multi-tenant never spawns llama-server). Deferred to avoid scope creep.

---

## Step 1: Create `devops/multi-tenant.Dockerfile` (AMD64)

**File**: `devops/multi-tenant.Dockerfile` (new file)

Copy `devops/cpu.Dockerfile` and modify. Key differences from `cpu.Dockerfile`:
- `BODHI_DEPLOYMENT: multi_tenant` (was `standalone`)
- `BODHI_PORT: 1135` (was `8080`)
- Hardcoded `BODHI_EXEC_TARGET: x86_64-unknown-linux-gnu` (no TARGETARCH case)
- `EXPOSE 1135` and healthcheck on port 1135

```dockerfile
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
# BodhiApp Default Configuration for Multi-Tenant (AMD64)
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
BODHI_EXEC_TARGET: x86_64-unknown-linux-gnu
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
```

---

## Step 2: Create `devops/multi-tenant-arm64.Dockerfile` (ARM64)

**File**: `devops/multi-tenant-arm64.Dockerfile` (new file)

Identical to `multi-tenant.Dockerfile` except:
- `BODHI_EXEC_TARGET: aarch64-unknown-linux-gnu` (was `x86_64-unknown-linux-gnu`)
- Comment says "ARM64" instead of "AMD64"

```dockerfile
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
```

---

## Step 3: Add Makefile targets in `devops/Makefile`

**File**: `devops/Makefile`

### 3a. Update `.PHONY` (line 5)

Add `build.multi-tenant build.multi-tenant-arm64 run.multi-tenant dev.multi-tenant dev.multi-tenant-arm64 prod.multi-tenant prod.multi-tenant-arm64` to the `.PHONY` line.

### 3b. Add build targets (after `build.vulkan`, line 110)

```makefile
build.multi-tenant: build.app-binary ## Build multi-tenant image (AMD64, for cloud)
	@echo "Building multi-tenant AMD64 image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/amd64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/multi-tenant.Dockerfile \
		-t bodhiapp:local-multi-tenant-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-multi-tenant-$(BUILD_VARIANT)"

build.multi-tenant-arm64: ## Build multi-tenant image (ARM64, for macOS dev)
	@$(MAKE) build.app-binary PLATFORM=linux/arm64
	@echo "Building multi-tenant ARM64 image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/arm64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/multi-tenant-arm64.Dockerfile \
		-t bodhiapp:local-multi-tenant-arm64-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-multi-tenant-arm64-$(BUILD_VARIANT)"
```

### 3c. Add run target (after `run.vulkan`, line 136)

Defaults to ARM64 variant since local dev is macOS/ARM64. Reads credentials from `devops/.env.local` via `docker run --env-file`:

```makefile
run.multi-tenant: ## Run multi-tenant image (defaults to ARM64 for macOS dev)
	@ENV_FILE="$(CURDIR)/.env.local"; \
	if [ ! -f "$$ENV_FILE" ]; then \
		echo "ERROR: $$ENV_FILE not found. Copy .env.example to .env.local and fill in values:"; \
		echo "  cp devops/.env.example devops/.env.local"; \
		exit 1; \
	fi; \
	VARIANT=$${VARIANT:-multi-tenant-arm64} && \
	BUILD_VAR=$${BUILD_VARIANT:-$(BUILD_VARIANT)} && \
	IMAGE_NAME="bodhiapp:local-$$VARIANT-$$BUILD_VAR"; \
	echo "Running BodhiApp multi-tenant container: $$IMAGE_NAME"; \
	mkdir -p $(PROJECT_ROOT)/docker-data/mt_bodhi_home $(PROJECT_ROOT)/docker-data/mt_hf_home && \
	docker run --rm -it \
		--env-file "$$ENV_FILE" \
		-p 1135:1135 \
		-v $(PROJECT_ROOT)/docker-data/mt_bodhi_home:/data/bodhi_home \
		-v $(PROJECT_ROOT)/docker-data/mt_hf_home:/data/hf_home \
		$$IMAGE_NAME
```

### 3d. Add dev/prod convenience targets (after line 176)

Local dev targets default to ARM64 (macOS). Explicit AMD64 targets for cloud builds:

```makefile
dev.multi-tenant: ## Quick development build for multi-tenant (ARM64, macOS dev)
	@$(MAKE) build.multi-tenant-arm64 BUILD_VARIANT=development

dev.multi-tenant.amd64: ## Quick development build for multi-tenant (AMD64, cloud)
	@$(MAKE) build.multi-tenant BUILD_VARIANT=development

prod.multi-tenant: ## Production build for multi-tenant (ARM64)
	@$(MAKE) build.multi-tenant-arm64 BUILD_VARIANT=production

prod.multi-tenant.amd64: ## Production build for multi-tenant (AMD64, cloud)
	@$(MAKE) build.multi-tenant BUILD_VARIANT=production
```

### 3e. Update help text (line 27)

Add `multi-tenant, multi-tenant-arm64` to the VARIANT description.

---

## Step 4: Add delegation targets in `Makefile.docker.mk`

**File**: `Makefile.docker.mk`

### 4a. Update `.PHONY` (line 4)

Add `docker.dev.multi-tenant docker.dev.multi-tenant.amd64 docker.run.multi-tenant`.

### 4b. Add targets (after `docker.dev.cuda`, line 18)

```makefile
docker.dev.multi-tenant: ## Build multi-tenant image (ARM64, macOS dev default)
	@$(MAKE) -C devops dev.multi-tenant BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.dev.multi-tenant.amd64: ## Build multi-tenant image (AMD64, cloud)
	@$(MAKE) -C devops dev.multi-tenant.amd64 BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.multi-tenant: ## Run multi-tenant image with sample env vars
	@$(MAKE) -C devops run.multi-tenant BUILD_VARIANT=$${BUILD_VARIANT:-development}
```

---

## Step 5: Create `docker/docker-compose.multi-tenant.yml`

**File**: `docker/docker-compose.multi-tenant.yml` (new file)

```yaml
# PostgreSQL containers for local multi-tenant development.
# App DB on port 44320, Session DB on port 44321.
# Uses different ports from dev (34320/34321) and test (54320/64320) containers.
#
# Usage:
#   docker compose -f docker/docker-compose.multi-tenant.yml up -d
#   make docker.dev.multi-tenant        # ARM64 (macOS default)
#   make docker.dev.multi-tenant.amd64  # AMD64 (cloud)
#   make docker.run.multi-tenant
services:
  bodhi_mt_app_db:
    image: postgres:17
    environment:
      POSTGRES_DB: bodhi_app
      POSTGRES_USER: bodhi_dev
      POSTGRES_PASSWORD: bodhi_dev
    ports:
      - "44320:5432"
    volumes:
      - mt_app_db_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bodhi_dev -d bodhi_app"]
      interval: 5s
      timeout: 5s
      retries: 5

  bodhi_mt_session_db:
    image: postgres:17
    environment:
      POSTGRES_DB: bodhi_sessions
      POSTGRES_USER: bodhi_dev
      POSTGRES_PASSWORD: bodhi_dev
    ports:
      - "44321:5432"
    volumes:
      - mt_session_db_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bodhi_dev -d bodhi_sessions"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  mt_app_db_data:
  mt_session_db_data:
```

---

## Step 6: Add multi-tenant variant to CI workflow matrix

**File**: `.github/workflows/publish-docker.yml`

Note: Only the AMD64 `multi-tenant` variant is built in CI. The ARM64 `multi-tenant-arm64` variant is local-only (the `build-app-binary` CI job only produces amd64 binaries).

### 6a. Add to matrix variant list (line 162-167)

```yaml
        variant:
          - cuda
          - rocm
          - vulkan
          - musa
          - intel
          - multi-tenant
```

### 6b. Add to matrix include block (after line 198)

```yaml
          - variant: multi-tenant
            platforms: 'linux/amd64'
            description: 'Multi-tenant SaaS deployment'
            docker_flags: '-p 1135:1135'
            hardware_req: 'AMD64 CPU with PostgreSQL'
            special_note: ''
```

### 6c. Add to release variant check (line 523)

```bash
VARIANTS=("cuda" "rocm" "vulkan" "musa" "intel" "multi-tenant")
```

**Image tags produced by CI:**
- `ghcr.io/.../bodhiapp:1.0.0-multi-tenant` (AMD64)
- `ghcr.io/.../bodhiapp:latest-multi-tenant` (AMD64)

---

## Step 7: Create `devops/TECHDEBT.md`

**File**: `devops/TECHDEBT.md` (new file)

```markdown
# devops — TECHDEBT

## Multi-Tenant Base Image

The `multi-tenant.Dockerfile` and `multi-tenant-arm64.Dockerfile` currently use
`ghcr.io/bodhisearch/llama.cpp:latest-cpu` as base image for structural consistency
with other variants. However, multi-tenant mode uses `MultitenantInferenceService`
which proxies to external LLM APIs — it never spawns a local llama.cpp process.

**Improvement**: Switch to `debian:bookworm-slim` as the base image. This would:
- Reduce image size by ~200MB (no llama-server binary or CPU-optimized libraries)
- Remove unused BODHI_EXEC_* and BODHI_LLAMACPP_* settings from defaults.yaml
- Require creating the `llama` user manually (currently provided by the llama.cpp base)
```

---

## Files Summary

| File | Change |
|------|--------|
| `devops/multi-tenant.Dockerfile` | **New** — AMD64 multi-tenant image, port 1135 |
| `devops/multi-tenant-arm64.Dockerfile` | **New** — ARM64 multi-tenant image, port 1135 |
| `devops/Makefile` | Add build/run/dev/prod targets for both variants |
| `Makefile.docker.mk` | Add delegation targets for both variants |
| `docker/docker-compose.multi-tenant.yml` | **New** — PostgreSQL on ports 44320/44321 |
| `.github/workflows/publish-docker.yml` | Add both variants to matrix + release check |
| `devops/TECHDEBT.md` | **New** — document slim base image opportunity |

---

## Runtime Environment Variables

When running either multi-tenant Docker image, deployers must provide:

| Variable | Required | Purpose |
|----------|----------|---------|
| `BODHI_APP_DB_URL` | Yes | PostgreSQL URL for app database |
| `BODHI_SESSION_DB_URL` | Yes | PostgreSQL URL for session database |
| `BODHI_MULTITENANT_CLIENT_ID` | Yes | Dashboard OAuth client ID |
| `BODHI_MULTITENANT_CLIENT_SECRET` | Yes | Dashboard OAuth client secret |
| `BODHI_ENCRYPTION_KEY` | Yes | Encryption key for secrets |
| `BODHI_PUBLIC_HOST` | Yes | Public hostname for OAuth callbacks |
| `BODHI_PUBLIC_PORT` | No | Public port (defaults to 1135) |
| `BODHI_PUBLIC_SCHEME` | No | Public scheme (defaults to http) |
| `BODHI_LOG_LEVEL` | No | Log level (defaults to info) |
| `BODHI_LOG_STDOUT` | No | Log to stdout (defaults to false) |

---

## Verification (Local — macOS/ARM64)

### 1. Build ARM64 image (default for local dev)
```bash
make docker.dev.multi-tenant
```

### 2. Start PostgreSQL
```bash
docker compose -f docker/docker-compose.multi-tenant.yml up -d
```

### 3. Run multi-tenant container
```bash
make docker.run.multi-tenant
```

### 4. Verify health check
```bash
curl http://localhost:1135/ping
```

### 5. Verify deployment mode
```bash
curl http://localhost:1135/api/v1/settings | jq '.[] | select(.key == "BODHI_DEPLOYMENT")'
# Should return: { "key": "BODHI_DEPLOYMENT", "value": "multi_tenant", "source": "system" }
```

### 6. Build AMD64 image (cloud target)
```bash
make docker.dev.multi-tenant.amd64
```
