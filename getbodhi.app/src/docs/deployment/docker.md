---
title: 'Docker Deployment'
description: 'Deploy Bodhi App with Docker - CPU, CUDA, and ROCm variants'
order: 401
---

# Docker Deployment

## Overview

Bodhi App provides optimized Docker images for different hardware configurations:

- **CPU**: Multi-platform (AMD64 + ARM64)
- **CUDA**: NVIDIA GPU acceleration (8-12x speedup)
- **ROCm**: AMD GPU acceleration
- **Vulkan**: Cross-vendor GPU acceleration

All variants run the same Bodhi App codebase with hardware-specific optimizations for llama.cpp inference.

**Docker Registry**: GitHub Container Registry (ghcr.io)

> **Note**: Use the GitHub CLI (`gh`) to explore available images and tags at [github.com/bodhisearch/Bodhi App/pkgs/container/bodhiapp](https://github.com/BodhiSearch/BodhiApp/pkgs/container/bodhiapp).

## Latest Docker Releases

For the most up-to-date Docker image versions and variants, visit [getbodhi.app](https://getbodhi.app). The website automatically displays the latest production Docker releases with copy-to-clipboard commands for all available variants.

**Why check the website:**

- Always shows the latest version numbers
- Automatically updates when new variants are released
- Provides ready-to-use docker pull commands
- No manual version checking required

The examples in this documentation use `latest-{variant}` tags for convenience, but you can find specific version tags (e.g., `0.0.2-cpu`) on the website for production deployments.

## Variant Comparison

| Variant    | Platforms    | Hardware         | Use Case                     | Performance       |
| ---------- | ------------ | ---------------- | ---------------------------- | ----------------- |
| **CPU**    | AMD64, ARM64 | Any CPU          | General purpose, ARM devices | Baseline          |
| **CUDA**   | AMD64        | NVIDIA GPU       | NVIDIA GPUs, cloud instances | 8-12x faster      |
| **ROCm**   | AMD64        | AMD GPU          | AMD GPUs                     | GPU accelerated\* |
| **Vulkan** | AMD64        | Cross-vendor GPU | Multi-vendor GPU support     | GPU accelerated\* |

> **Note**: Performance benchmark data is not yet available for ROCm and Vulkan variants. For image sizes, use `gh` CLI to query the container registry. New variants may be added over time - check [getbodhi.app](https://getbodhi.app) for the complete list.

**Choosing a Variant**:

1. **Have NVIDIA GPU?** → CUDA variant (best performance)
2. **Have AMD GPU?** → ROCm variant
3. **Need cross-vendor GPU support?** → Vulkan variant
4. **CPU only or ARM device?** → CPU variant

## Prerequisites

- Docker 20.10+ installed ([installation guide](https://docs.docker.com/get-docker/))

**For GPU Variants**:

- **CUDA**: NVIDIA GPU with CUDA 11+ support, Docker with NVIDIA GPU support ([installation guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html))
- **ROCm**: AMD GPU with ROCm support (refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for requirements)

## Quick Start

### CPU Variant (Recommended for Most Users)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu

# Run container
docker run -d \
  -p 1135:1135 \
  -v bodhi-data:/data \
  -v bodhi-models:/models \
  --name bodhiapp \
  ghcr.io/bodhisearch/bodhiapp:latest-cpu
```

**Access**: Open browser to `http://localhost:1135`

### CUDA Variant (NVIDIA GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cuda

# Run container with GPU access
docker run -d \
  -p 1135:1135 \
  -v bodhi-data:/data \
  -v bodhi-models:/models \
  --gpus all \
  --name bodhiapp-cuda \
  ghcr.io/bodhisearch/bodhiapp:latest-cuda
```

> **Note**: The Docker images use base images from GPU vendors with required runtime libraries included. Use the `--gpus all` flag to provide GPU access to the container.

### ROCm Variant (AMD GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-rocm

# Run container with GPU access
docker run -d \
  -p 1135:1135 \
  -v bodhi-data:/data \
  -v bodhi-models:/models \
  --device=/dev/kfd \
  --device=/dev/dri \
  --name bodhiapp-rocm \
  ghcr.io/bodhisearch/bodhiapp:latest-rocm
```

> **Note**: For AMD GPU device mapping, refer to [llama.cpp ROCm documentation](https://github.com/ggerganov/llama.cpp) for specific requirements.

## Volume Configuration

### Required Volumes

**Data Volume** (`/data`):

- Configuration files
- Database (users, tokens, requests)
- Application settings
- Logs

**Models Volume** (`/models`):

- Downloaded GGUF models
- Model cache
- Model aliases (stored as files)
- SQLite-based application state

> **Volume Size**: Depends on the models you download. Allocate space based on your model requirements (typically 4-80GB per model). Also includes minimal space for SQLite database and alias files.

### Volume Examples

**Named Volumes** (Recommended):

```bash
# Create named volumes
docker volume create bodhi-data
docker volume create bodhi-models

# Use in run command
docker run -v bodhi-data:/data -v bodhi-models:/models ...
```

**Bind Mounts** (Direct host paths):

```bash
# Use host directories
docker run \
  -v /path/to/data:/data \
  -v /path/to/models:/models \
  ...
```

## Environment Variables

### Essential Configuration

**Common Environment Variables**:

```bash
# Server Configuration
-e BODHI_PORT=1135 \              # Server port (default: 1135)
-e BODHI_HOST=0.0.0.0 \           # Server host (default: 0.0.0.0)
-e BODHI_ENCRYPTION_KEY=your-key \ # Required for data encryption

# RunPod Auto-Configuration
-e BODHI_ON_RUNPOD=true \         # Enables RunPod-specific auto-config

# Public Host (for cloud/network deployments)
-e BODHI_PUBLIC_SCHEME=https \
-e BODHI_PUBLIC_HOST=your-domain.com \
-e BODHI_PUBLIC_PORT=443 \
```

> **Note**: `BODHI_ENCRYPTION_KEY` is required for securing stored data. For complete environment variable reference, see [Configuration Guide](/docs/developer/configuration) (coming soon).

## Cloud Platform Deployment

> **Note**: Railway-specific deployment is not yet supported. Use Docker deployment on your preferred cloud platform.

### RunPod

**RunPod Auto-Configuration**:
Bodhi App supports automatic configuration for RunPod deployments. Set the `BODHI_ON_RUNPOD=true` environment variable to enable auto-configuration using RunPod-injected environment variables for public host and other properties.

**Steps**:

1. Create new pod on RunPod
2. Select Docker variant:
   - `ghcr.io/bodhisearch/bodhiapp:latest-cuda` for GPU pods
   - `ghcr.io/bodhisearch/bodhiapp:latest-cpu` for CPU pods
3. Configure volumes (`/data` and `/models`)
4. Set environment variables (including `BODHI_ON_RUNPOD=true`)
5. Deploy

### Generic Cloud Platform

Bodhi App works on any platform supporting Docker:

**Requirements**:

- Docker support
- Volume/storage support
- Network ingress
- **Minimum Resources** (API-only, lightweight workflow):
  - 2GB RAM
  - Single-core 2.4GHz CPU
- **For Local Model Inference**: Resources depend on the model size and requirements

**Configuration**:

1. Deploy Docker image
2. Configure volumes (`/data` and `/models`)
3. Set public host environment variables
4. Configure OAuth callback URL
5. Open port 1135 (or custom port)

## Docker Compose

> **Note**: Docker Compose deployment has not been tested. Use single-container deployment commands shown above.

## Performance Optimization

### GPU Configuration

**Docker Factory Settings**:

- Docker files include factory settings optimized for fastest single-request inference
- Settings can be overridden via environment variables or the settings dashboard
- All settings are llama.cpp pass-through parameters

**CUDA Requirements**:

- NVIDIA GPU with CUDA 11+ support
- Docker with NVIDIA GPU support ([installation guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html))
- Use `--gpus all` flag for GPU access

**ROCm**: Check [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for specific requirements.

### Parallel Request Optimization

For handling parallel requests, override factory settings (optimized for single requests) using:

- Environment variables
- Settings dashboard configuration
- Refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for optimal settings for your hardware

> **Note**: Performance benchmark data is not yet available.

## Troubleshooting

### Container Won't Start

**Symptoms**: Container exits immediately or won't start

**Common Issues**:

- **Missing `BODHI_ENCRYPTION_KEY`**: This environment variable is required for data encryption
- **Port conflicts**: Application runs on port 1135 by default. If port is unavailable, the application fails with an error message

**Solutions**:

- Check logs: `docker logs <container-name>`
- Verify `BODHI_ENCRYPTION_KEY` is set
- Check environment variables are correctly configured
- Verify volume permissions
- Search the codebase for error codes to understand specific issues

### GPU Not Detected (CUDA)

**Symptoms**: Running but using CPU instead of GPU

**Solutions**:

- Verify NVIDIA Docker runtime installed
- Check `--gpus all` flag is used in docker run command
- Run `nvidia-smi` in container to verify GPU is visible
- Verify CUDA 11+ support

### GPU Not Detected (ROCm)

**Symptoms**: AMD GPU not utilized

**Solutions**:

- Verify correct device mapping flags are used
- Check [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for ROCm requirements
- Verify driver installation

### Performance Slower Than Expected

**Symptoms**: Inference speed below expectations

**Possible Causes & Solutions**:

- **Factory settings for single request**: If running parallel requests, override settings via environment variables or settings dashboard
- **Model inference powered by llama.cpp**: Refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for optimal configuration
- **Wrong hardware variant**: Ensure using appropriate Docker variant for your hardware (CUDA for NVIDIA, ROCm for AMD, etc.)

### OAuth Redirect Issues

**Symptoms**: OAuth callback fails in cloud deployment

**Solutions**:

- Verify BODHI_PUBLIC_HOST matches actual domain
- Set BODHI_PUBLIC_SCHEME to https (if using HTTPS)
- Set BODHI_PUBLIC_PORT correctly
- Update OAuth callback URL in provider
- See [OAuth Configuration Guide](/docs/intro#authentication)

## Upgrading

### Upgrade Process

```bash
# Stop container
docker stop bodhiapp

# Pull new image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu

# Remove old container (keeps volumes)
docker rm bodhiapp

# Start new container with same config
docker run -d \
  -p 1135:1135 \
  -v bodhi-data:/data \
  -v bodhi-models:/models \
  --name bodhiapp \
  ghcr.io/bodhisearch/bodhiapp:latest-cpu
```

**Data Safety**: Volumes are preserved. Database migrations run automatically on startup.

### Backup and Restore

**Backup Strategy**:
To backup your Bodhi App installation, you need to save:

1. **BODHI_HOME folder** (or the `/data` volume) - Contains configuration, database, and application state
2. **BODHI_ENCRYPTION_KEY environment variable** - Required for data decryption

Both must match during restore for the application to work correctly.

**Backup Commands**:

```bash
# Backup data volume
docker run --rm \
  -v bodhi-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/bodhi-data-backup.tar.gz /data

# Backup models volume
docker run --rm \
  -v bodhi-models:/models \
  -v $(pwd):/backup \
  alpine tar czf /backup/bodhi-models-backup.tar.gz /models
```

### Version Pinning

**Latest Tag**:

```bash
# Always latest version (not recommended for production)
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu
```

**Specific Version**:

```bash
# Pin to specific version (recommended for production)
docker pull ghcr.io/bodhisearch/bodhiapp:v0.1.0-cpu
```

> **Note**: Use `gh` CLI to explore available version tags at the container registry.

## Hardware Acceleration Details

### CUDA Variant

**Supported GPUs**:

- NVIDIA GPUs with CUDA Compute Capability 5.0+ (Maxwell and newer)
- CUDA 11 and 12 support

**Performance Gains**:

- 8-12x speedup vs CPU for typical models
- Performance varies by model size

**VRAM Requirements**:

- Varies based on model size
- Refer to HuggingFace model page for specific llama.cpp hardware requirements

**GPU Recommendations**:

- Bodhi App recommends optimal models during setup based on open-source benchmarks
- No dynamic recommendations currently available

**Configuration**:

```bash
# Check GPU availability in container
docker exec -it bodhiapp-cuda nvidia-smi

# Monitor GPU usage
nvidia-smi -l 1
```

### ROCm Variant

**Supported GPUs**:

- AMD Radeon RX and Instinct series
- Refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for ROCm support details

**VRAM Requirements**:

- Varies based on model size
- Refer to HuggingFace model page for specific llama.cpp hardware requirements

### CPU Variant

**Optimizations**:

- Multi-threaded inference with configurable thread count
- AVX/AVX2/AVX512 instruction set support (when available)
- ARM NEON optimizations for ARM64

**Performance Tips**:

- For optimal settings, refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp)
- Run on dedicated hardware for best performance

## Security Considerations

### Container Security

**Best Practices**:

- Run containers with non-root user when possible
- Use read-only root filesystem where feasible
- Limit container capabilities
- Keep images updated with latest security patches

**Network Security**:

- Use reverse proxy (nginx, Traefik) for HTTPS termination
- Configure firewall rules appropriately
- Use Docker networks for service isolation

### Secrets Management

**OAuth Credentials**:

- Use environment variables for sensitive configuration
- Consider Docker secrets or external secret management (Vault, AWS Secrets Manager)
- Never commit credentials to version control

**API Keys**:

- Store remote AI API keys as environment variables
- Rotate keys regularly
- Use separate keys for development and production

## Resource Planning

### Storage Requirements

**Data Volume**:

- Initial: ~100MB (configuration and database)
- Growth: Minimal (user data, chat history in LocalStorage)

**Models Volume**:

- Small models (7B quantized): 4-8GB each
- Medium models (13B quantized): 8-16GB each
- Large models (70B quantized): 40-80GB each

**Total Recommendations**:

- Light usage (1-2 models): 50GB
- Medium usage (3-5 models): 150GB
- Heavy usage (10+ models): 500GB+

### Memory and CPU Requirements

**Minimum Requirements** (API-only, lightweight workflow):

- 2GB RAM
- Single-core 2.4GHz CPU

**For Local Model Inference**:

- **CPU and Memory**: Requirements depend on model size
- **VRAM**: Varies by model size - refer to HuggingFace model page for specific llama.cpp requirements
- **GPU Variants**: CPU usage is reduced when GPU is active

## Monitoring and Observability

### Logs

**Access Container Logs**:

```bash
# View logs
docker logs bodhiapp

# Follow logs in real-time
docker logs -f bodhiapp

# Last 100 lines
docker logs --tail 100 bodhiapp
```

**Log Levels**:

- Configure via `BODHI_LOG_LEVEL` environment variable
- For available log levels and configuration details, see [Configuration Guide](/docs/developer/configuration) (coming soon)

### Health Checks

**Manual Health Check**:

```bash
# Check if server responds
curl http://localhost:1135/health

# Check from inside container
docker exec bodhiapp curl http://localhost:1135/health
```

> **Note**: For health check endpoint details, see the OpenAPI documentation.

### Performance Monitoring

**Resource Usage**:

```bash
# Container resource stats
docker stats bodhiapp

# Detailed inspection
docker inspect bodhiapp
```

**GPU Monitoring** (CUDA):

```bash
# Inside container
docker exec -it bodhiapp-cuda nvidia-smi

# Continuous monitoring
docker exec -it bodhiapp-cuda nvidia-smi -l 1
```

## Multi-Container Deployments

> **Note**: For multi-container deployments, load balancing, high availability configurations, and session persistence details, see [Advanced Deployment Guide](/docs/deployment/advanced) (coming soon).

## Migration and Backup

### Backup Strategy

**Data Backup**:

```bash
# Backup data volume
docker run --rm \
  -v bodhi-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/bodhi-data-backup.tar.gz /data

# Backup models volume
docker run --rm \
  -v bodhi-models:/models \
  -v $(pwd):/backup \
  alpine tar czf /backup/bodhi-models-backup.tar.gz /models
```

**Restore**:

```bash
# Restore data volume
docker run --rm \
  -v bodhi-data:/data \
  -v $(pwd):/backup \
  alpine sh -c "cd / && tar xzf /backup/bodhi-data-backup.tar.gz"

# Restore models volume
docker run --rm \
  -v bodhi-models:/models \
  -v $(pwd):/backup \
  alpine sh -c "cd / && tar xzf /backup/bodhi-models-backup.tar.gz"
```

### Migration Between Hosts

**Steps**:

1. Stop container on source host
2. Backup volumes (see above)
3. Transfer backup files to destination host
4. Restore volumes on destination host
5. Start container with same configuration

> **Note**: For database portability and migration details, see [Configuration Guide](/docs/developer/configuration) (coming soon).

## Related Documentation

- [Installation Guide](/docs/install) - Desktop and server installation
- [Environment Variables](/docs/features/app-settings) - Complete configuration reference
- [Authentication](/docs/intro#authentication) - OAuth2 setup
- [Multi-Platform Installation](/docs/deployment/platforms) - Desktop apps
- [llama.cpp Documentation](https://github.com/ggerganov/llama.cpp) - GPU requirements and performance tuning
