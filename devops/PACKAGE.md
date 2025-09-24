# PACKAGE.md

See [CLAUDE.md](./CLAUDE.md) for architectural documentation and design rationale.

## Implementation Overview

The `devops` folder provides BodhiApp's containerized deployment infrastructure with multi-platform Docker build support, hardware-optimized configurations, and comprehensive build automation.

## Core Files

### Build Orchestration
- `devops/Makefile:1-150` - Complete Docker build system with variants, platforms, and lifecycle management
  ```makefile
  # Multi-platform CPU build example
  build.cpu: ## Build CPU image for current platform
    PLATFORM=$${PLATFORM:-linux/amd64} && \
    cd $(PROJECT_ROOT) && docker buildx build --load \
      --platform $$PLATFORM --build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
      -f devops/cpu.Dockerfile -t bodhiapp:local-cpu-$(BUILD_VARIANT) .
  ```

### Docker Configurations

#### CPU Deployment
- `devops/cpu.Dockerfile:1-163` - Multi-stage CPU-only build with llama.cpp base image integration
  ```dockerfile
  # Multi-stage build pattern for CPU optimization
  FROM ghcr.io/bodhisearch/llama.cpp:latest-cpu AS runtime
  FROM rust:1.87.0-bookworm AS builder
  # Dependency caching, TS client build, application compilation stages
  ```

#### GPU Acceleration Variants
- `devops/cuda.Dockerfile:1-163` - NVIDIA CUDA GPU acceleration with performance optimizations
- `devops/rocm.Dockerfile:1-163` - AMD ROCm GPU support with HIP compute integration  
- `devops/vulkan.Dockerfile:1-163` - Cross-vendor Vulkan GPU acceleration

### Documentation
- `devops/README.md:1-110` - User-facing quick start guide and troubleshooting
- `devops/CLAUDE.md:1-110` - Architectural documentation and cross-system integration patterns

## Key Implementation Patterns

### Multi-Stage Docker Build Strategy
Each Dockerfile implements a consistent 3-stage build pattern:

1. **Dependency Caching Stage** (`devops/cpu.Dockerfile:44-62`)
   ```dockerfile
   # Filter Cargo.toml for dependency-only build
   RUN python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml
   # Pre-compile dependencies with optimization consistency
   RUN cargo build --release -p ci_optims
   ```

2. **TypeScript Client Build** (`devops/cpu.Dockerfile:64-68`)
   ```dockerfile
   COPY ts-client/ ts-client/
   RUN cd ts-client && npm ci && npm run build:docker
   ```

3. **Application Compilation** (`devops/cpu.Dockerfile:70-94`)
   ```dockerfile
   # Build with CI_BUILD_TARGET for platform targeting
   RUN cargo build --release --bin bodhi --no-default-features --features production
   ```

### Hardware-Specific Configuration
Runtime configuration injection based on target platform:

```dockerfile
# Platform-specific target architecture detection
RUN case "$TARGETARCH" in \
    "arm64") export EXEC_TARGET="aarch64-unknown-linux-gnu" ;; \
    "amd64"|*) export EXEC_TARGET="x86_64-unknown-linux-gnu" ;; \
  esac
```

### Build Variant Management
Development and production build optimization handling:

```makefile
# Build variant selection with consistent optimization levels
RUN if [ "$BUILD_VARIANT" = "production" ]; then \
    cargo build --release -p ci_optims; \
  else \
    cargo build -p ci_optims; \
  fi
```

## Build System Integration

### Root Makefile Delegation
Target delegation from project root (`Makefile:192-214`):

```makefile
docker.dev.cpu: ## Build CPU image for current platform
  @$(MAKE) -C devops dev.cpu BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.amd64: ## Run locally built linux/amd64 Docker image  
  @$(MAKE) -C devops run VARIANT=$${VARIANT:-cpu} ARCH=$${ARCH:-amd64}
```

### Environment Variable Integration
Runtime configuration through environment variables (`devops/Makefile:86-101`):

```makefile
# Container runtime configuration
docker run --rm -it \
  -e BODHI_LOG_STDOUT=true \
  -e BODHI_ENCRYPTION_KEY=local-dev-key \
  -p 8080:8080 \
  -v $(PROJECT_ROOT)/docker-data/bodhi_home:/data/bodhi_home
```

## Hardware Acceleration Configurations

### CUDA Optimization (`devops/cuda.Dockerfile:135-154`)
```yaml
# GPU-accelerated server arguments in defaults.yaml
BODHI_LLAMACPP_ARGS_CUDA: "--flash-attn --n-gpu-layers 99 --batch-size 512 --ubatch-size 512 --cache-type-k q4_0 --cache-type-v q4_0 --threads 1"
```

### ROCm Integration (`devops/rocm.Dockerfile:135-154`) 
```yaml
# AMD GPU configuration with row-split support
BODHI_LLAMACPP_ARGS_ROCM: "--n-gpu-layers 99 --split-mode row --batch-size 512 --ubatch-size 512 --threads 1"
```

## Development Commands

### Local Development Workflow
```bash
# Build development variant (default)
make docker.dev.cpu.amd64

# Build production optimized
make docker.dev.cpu.amd64 BUILD_VARIANT=production

# Run with GPU support  
make docker.dev.cuda
make docker.run.amd64 VARIANT=cuda

# Cleanup
make docker.clean
```

### Multi-Platform Building
```bash
# Cross-platform builds (requires Docker Buildx)
make build.cpu.multi                    # AMD64 + ARM64
make build.cpu PLATFORM=linux/arm64     # Specific platform
```

### Image Management
```bash
make list-images                        # List all built images
make clean                             # Remove all local images  
make help                              # Show all available targets
```

## Container Runtime Integration

### Volume Mounting Strategy
Persistent data management (`devops/Makefile:91-100`):
- `/data/bodhi_home` - Application configuration and data
- `/data/hf_home` - HuggingFace model cache
- Local development: `docker-data/` directory structure

### Health Check Integration
HTTP-based container health monitoring (`devops/cpu.Dockerfile:157-159`):
```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/ping || exit 1
```

### Security Model
Non-root container execution with proper ownership (`devops/cpu.Dockerfile:98-151`):
```dockerfile
USER root
# File operations and ownership setup
RUN chown llama:llama /app/bodhi && chmod +x /app/bodhi
USER llama  # Switch back to non-privileged user
```

## CI/CD Integration Points

### Version Information Injection
Build-time metadata embedding:
```dockerfile
ARG BODHI_VERSION
ARG BODHI_COMMIT_SHA
# Injected into defaults.yaml for runtime identification
```

### Platform Detection
Architecture-specific build targeting:
```dockerfile
ARG TARGETARCH
# Used for CI_BUILD_TARGET selection and executable targeting
```

### Base Image Dependencies
Specialized base images for hardware optimization:
- `ghcr.io/bodhisearch/llama.cpp:latest-cpu` - CPU-optimized runtime
- `ghcr.io/bodhisearch/llama.cpp:latest-cuda` - NVIDIA CUDA support
- `ghcr.io/bodhisearch/llama.cpp:latest-rocm` - AMD ROCm integration
- `ghcr.io/bodhisearch/llama.cpp:latest-vulkan` - Cross-vendor GPU support