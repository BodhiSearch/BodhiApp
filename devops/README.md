# BodhiApp Docker Build System

This directory contains the Docker build infrastructure for BodhiApp, including Dockerfiles for all variants and a comprehensive Makefile for local development.

## Quick Start

```bash
# From project root - use delegated targets
make docker.build.cpu.amd64              # Build AMD64 CPU image
make docker.run VARIANT=cpu ARCH=amd64   # Run the built image

# From devops/ directory - use direct targets  
cd devops
make build.cpu.amd64                     # Build AMD64 CPU image
make run VARIANT=cpu ARCH=amd64          # Run the built image
make help                                # Show all available targets
```

## Available Docker Variants

### CPU Variants
- **AMD64 CPU** (`cpu.Dockerfile`): Uses llama.cpp base image, fast build
- **ARM64 CPU** (`cpu-arm64.Dockerfile`): Downloads binaries, requires `GH_PAT`

### GPU Variants (AMD64 only)
- **CUDA** (`cuda.Dockerfile`): NVIDIA GPU acceleration with optimized llama-server flags
- **ROCm** (`rocm.Dockerfile`): AMD GPU acceleration  
- **Vulkan** (`vulkan.Dockerfile`): Cross-vendor GPU acceleration

## Build Variants

- **Development** (default): Faster builds, debug symbols
- **Production**: Optimized builds, smaller images

```bash
make build.cpu.amd64                           # Development build (default)
make build.cpu.amd64 BUILD_VARIANT=production # Production build
```

## Local Development Workflow

```bash
# 1. Build an image
make docker.build.cpu.amd64

# 2. Run it locally
make docker.run VARIANT=cpu ARCH=amd64

# 3. Build GPU variant (if you have compatible hardware)
make docker.build.cuda

# 4. Run GPU variant
make docker.run VARIANT=cuda

# 5. List built images
make docker.list

# 6. Clean up when done
make docker.clean
```

## Architecture

- **Root Makefile**: Simple delegation to devops/ targets for convenience
- **devops/Makefile**: Comprehensive build logic for all Docker variants
- **Dockerfiles**: Each variant has its own optimized Dockerfile
- **GitHub Workflow**: CI/CD handles multi-platform builds and publishing

## Requirements

- Docker with buildx support
- For ARM64 builds: `GH_PAT` environment variable
- For GPU variants: Compatible GPU drivers

## Troubleshooting

**"No rule to make target 'docker.build.*'"**: 
- Ensure you're running from project root or use `make -C devops <target>`

**ARM64 build fails**:
- Set `GH_PAT` environment variable with GitHub personal access token

**GPU builds fail**:
- Ensure you have compatible GPU drivers installed
- GPU variants only support AMD64 architecture

## CUDA Optimization

The CUDA variant includes comprehensive performance optimizations for NVIDIA GPUs:

- **Flash Attention**: Memory-efficient attention computation for 15-27% speedup
- **Full GPU Offloading**: All model layers moved to GPU for maximum performance
- **Optimized Batching**: Large batch sizes for improved throughput
- **KV Cache Quantization**: 50% memory savings with minimal quality impact
- **Threading Optimization**: Prevents CPU bottlenecks in hybrid operations

For detailed information on CUDA optimizations, see: [`ai-docs/context/cuda-Dockerfile-optimizations.md`](../ai-docs/context/cuda-Dockerfile-optimizations.md)

Expected performance improvements:
- **8-12x token generation speed** compared to unoptimized configurations
- **25-35 tokens/second** on RTX A4000 16GB with 14B parameter models
- **Efficient VRAM usage** (~12-13GB for 14B Q4_K_M models)

## Migration from Old System

Old targets have been cleaned up:
- ~~`docker.build`~~ → `docker.build.cpu.amd64`
- ~~`docker.build.dev`~~ → `docker.build.cpu.amd64` (development is now default)
- ~~`docker.build.optimized`~~ → `docker.build.cpu.amd64 BUILD_VARIANT=production`
- ~~`docker.build.multi`~~ → Handled by CI/CD workflow