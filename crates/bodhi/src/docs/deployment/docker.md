---
title: 'Docker Deployment'
description: 'Deploy Bodhi App with Docker - CPU, CUDA, ROCm, Vulkan, MUSA, Intel, and CANN variants'
order: 401
---

# Docker Deployment

## Overview

Bodhi App provides optimized Docker images for different hardware configurations:

- **CPU**: Multi-platform (AMD64 + ARM64)
- **CUDA**: NVIDIA GPU acceleration
- **ROCm**: AMD GPU acceleration
- **Vulkan**: Cross-vendor GPU acceleration
- **MUSA**: Moore Threads GPU acceleration
- **Intel**: Intel GPU acceleration (SYCL)
- **CANN**: Huawei Ascend NPU acceleration

All variants run the same Bodhi App codebase with hardware-specific optimizations for llama.cpp inference. The internal server listens on port **8080**, which you map to a host port (e.g., 1135) via `-p`.

**Docker Registry**: GitHub Container Registry (ghcr.io)

> **Note**: Use the GitHub CLI (`gh`) to explore available images and tags at [github.com/bodhisearch/BodhiApp/pkgs/container/bodhiapp](https://github.com/BodhiSearch/BodhiApp/pkgs/container/bodhiapp).

## Latest Docker Releases

For the most up-to-date Docker image versions and variants, visit [getbodhi.app](https://getbodhi.app). The website automatically displays the latest production Docker releases with copy-to-clipboard commands for all available variants.

**Why check the website:**

- Always shows the latest version numbers
- Automatically updates when new variants are released
- Provides ready-to-use docker pull commands
- No manual version checking required

The examples in this documentation use `latest-{variant}` tags for convenience, but you can find specific version tags (e.g., `0.0.2-cpu`) on the website for production deployments.

## Variant Comparison

| Variant    | Platforms         | Hardware             | Use Case                         |
| ---------- | ----------------- | -------------------- | -------------------------------- |
| **CPU**    | AMD64, ARM64      | Any CPU              | General purpose, ARM devices     |
| **CUDA**   | AMD64             | NVIDIA GPU           | NVIDIA GPUs, cloud instances     |
| **ROCm**   | AMD64             | AMD GPU              | AMD Radeon / Instinct GPUs       |
| **Vulkan** | AMD64             | Cross-vendor GPU     | Multi-vendor GPU support         |
| **MUSA**   | AMD64             | Moore Threads GPU    | Moore Threads S-series GPUs      |
| **Intel**  | AMD64             | Intel GPU            | Intel Arc / Data Center GPUs     |
| **CANN**   | AMD64, ARM64      | Huawei Ascend NPU   | Huawei Ascend AI processors      |

> **Note**: Check [getbodhi.app](https://getbodhi.app) for the complete and up-to-date list of available variants.

**Choosing a Variant**:

1. **Have NVIDIA GPU?** -> CUDA variant (best performance)
2. **Have AMD GPU?** -> ROCm variant
3. **Have Intel GPU?** -> Intel variant
4. **Have Moore Threads GPU?** -> MUSA variant
5. **Have Huawei Ascend NPU?** -> CANN variant
6. **Need cross-vendor GPU support?** -> Vulkan variant
7. **CPU only or ARM device?** -> CPU variant

## Prerequisites

- Docker 20.10+ installed ([installation guide](https://docs.docker.com/get-docker/))

**For GPU Variants**:

- **CUDA**: NVIDIA GPU with CUDA 11+ support, Docker with NVIDIA GPU support ([installation guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html))
- **ROCm**: AMD GPU with ROCm support (refer to [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for requirements)
- **Vulkan**: GPU with Vulkan driver support
- **MUSA**: Moore Threads GPU with MUSA toolkit
- **Intel**: Intel GPU with oneAPI / SYCL support
- **CANN**: Huawei Ascend NPU with CANN toolkit

## Quick Start

### CPU Variant (Recommended for Most Users)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu

# Run container
docker run --name bodhiapp \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  ghcr.io/bodhisearch/bodhiapp:latest-cpu
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

**Access**: Open browser to `http://localhost:1135`

### CUDA Variant (NVIDIA GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cuda

# Run container with GPU access
docker run --name bodhiapp-cuda \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --gpus all \
  ghcr.io/bodhisearch/bodhiapp:latest-cuda
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

> **Note**: The Docker images use base images from GPU vendors with required runtime libraries included. Use the `--gpus all` flag to provide GPU access to the container.

### ROCm Variant (AMD GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-rocm

# Run container with GPU access
docker run --name bodhiapp-rocm \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --device=/dev/kfd \
  --device=/dev/dri \
  ghcr.io/bodhisearch/bodhiapp:latest-rocm
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

### Vulkan Variant (Cross-Vendor GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-vulkan

# Run container with GPU access
docker run --name bodhiapp-vulkan \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --device=/dev/dri \
  ghcr.io/bodhisearch/bodhiapp:latest-vulkan
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

### MUSA Variant (Moore Threads GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-musa

# Run container with GPU access
docker run --name bodhiapp-musa \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --device=/dev/mthreads \
  ghcr.io/bodhisearch/bodhiapp:latest-musa
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

### Intel Variant (Intel GPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-intel

# Run container with GPU access
docker run --name bodhiapp-intel \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --device=/dev/dri \
  ghcr.io/bodhisearch/bodhiapp:latest-intel
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

### CANN Variant (Huawei Ascend NPU)

```bash
# Pull image
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cann

# Run container with NPU access
docker run --name bodhiapp-cann \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
  -v $(pwd)/docker-data:/data \
  --device=/dev/davinci0 \
  --device=/dev/davinci_manager \
  --device=/dev/devmm_svm \
  --device=/dev/hisi_hdc \
  ghcr.io/bodhisearch/bodhiapp:latest-cann
```

> **Important**: Replace `your-strong-encryption-key-here` with your own strong encryption key. The container validates the encryption key on startup and will not start with the placeholder value.

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
-e BODHI_HOST=0.0.0.0 \           # Server host (default: 0.0.0.0)
-e BODHI_ENCRYPTION_KEY=your-key \ # Required for data encryption

# RunPod Auto-Configuration
-e BODHI_ON_RUNPOD=true \         # Enables RunPod-specific auto-config

# Public Host (for cloud/network deployments)
-e BODHI_PUBLIC_SCHEME=https \
-e BODHI_PUBLIC_HOST=your-domain.com \
-e BODHI_PUBLIC_PORT=443 \
```

> **Note**: `BODHI_ENCRYPTION_KEY` is required for securing stored data. The container validates the encryption key on startup and will refuse to start if it is missing or invalid.

### Deployment Mode

`BODHI_DEPLOYMENT` is an immutable build-time property baked into each Docker image. Standalone images (all GPU variants above) use `BODHI_DEPLOYMENT=standalone` with SQLite. Multi-tenant images use `BODHI_DEPLOYMENT=multi_tenant` with PostgreSQL and row-level security (RLS). You cannot change the deployment mode at runtime.

## Cloud Platform Deployment

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
5. Open the host port you mapped to container port 8080

## Docker Compose

```yaml
version: '3.8'
services:
  bodhiapp:
    image: ghcr.io/bodhisearch/bodhiapp:latest-cpu
    ports:
      - '1135:8080'
    environment:
      - BODHI_PUBLIC_HOST=0.0.0.0
      - BODHI_PUBLIC_PORT=1135
      - BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here
    volumes:
      - bodhi-data:/data
      - bodhi-models:/models

volumes:
  bodhi-data:
  bodhi-models:
```

For GPU variants, add the appropriate device flags. For example, with CUDA:

```yaml
services:
  bodhiapp:
    image: ghcr.io/bodhisearch/bodhiapp:latest-cuda
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
    # ... rest of config same as above
```

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

## Troubleshooting

### Container Won't Start

**Symptoms**: Container exits immediately or won't start

**Common Issues**:

- **Missing or invalid `BODHI_ENCRYPTION_KEY`**: This environment variable is required. The container validates it on startup and will exit if it is missing or set to the placeholder value
- **Port conflicts**: The container exposes port 8080 internally. If the host port you map to is already in use, Docker will report a bind error

**Solutions**:

- Check logs: `docker logs <container-name>`
- Verify `BODHI_ENCRYPTION_KEY` is set to a real key
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

- Verify correct device mapping flags are used (`--device=/dev/kfd --device=/dev/dri`)
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
- See [Authentication](/docs/intro#authentication)

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
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=0.0.0.0 \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here \
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

**Restore Commands**:

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

**Factory Optimizations** (baked into defaults.yaml):

- Flash attention enabled
- Full GPU offloading (`--n-gpu-layers -1`)
- KV cache quantization (`q8_0`) for reduced VRAM usage
- Optimized batch sizes (2048/512)
- Thread pinning (8 threads)
- Memory locking enabled

**VRAM Requirements**:

- Varies based on model size
- Refer to HuggingFace model page for specific llama.cpp hardware requirements

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

**Factory Optimizations**:

- Full GPU offloading (`--n-gpu-layers 999`)
- Row-split memory management (`--split-mode row`)
- HIP compute integration (`--hipblas`)

### Vulkan Variant

**Supported GPUs**:

- Any GPU with Vulkan driver support (NVIDIA, AMD, Intel)

**Factory Optimizations**:

- GPU offloading (`--n-gpu-layers 999`)
- Vulkan compute backend enabled

### MUSA Variant

**Supported GPUs**:

- Moore Threads S-series GPUs with MUSA toolkit

**Factory Optimizations**:

- Full GPU offloading (`--n-gpu-layers -1`)
- Optimized batch sizes (2048/512)
- Memory locking enabled

### Intel Variant

**Supported GPUs**:

- Intel Arc discrete GPUs
- Intel Data Center GPUs

**Factory Optimizations**:

- Full GPU offloading (`--n-gpu-layers -1`)
- SYCL compute backend
- Optimized batch sizes (2048/512)
- Memory locking enabled

### CANN Variant

**Supported Hardware**:

- Huawei Ascend 310/910 series NPUs
- Supports both AMD64 and ARM64 platforms

**Factory Optimizations**:

- Full NPU offloading (`--n-gpu-layers -1`)
- Optimized batch sizes (2048/512)
- Memory locking enabled

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

- All containers run as non-root `llama` user by default
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

### Health Checks

**Manual Health Check**:

```bash
# Check if server responds
curl http://localhost:1135/ping

# Check from inside container
docker exec bodhiapp curl http://localhost:8080/ping
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

## Migration Between Hosts

**Steps**:

1. Stop container on source host
2. Backup volumes (see Backup and Restore above)
3. Transfer backup files to destination host
4. Restore volumes on destination host
5. Start container with same configuration

## Related Documentation

- [Installation Guide](/docs/install) - Desktop and server installation
- [App Settings](/docs/features/settings/app-settings) - Application settings reference
- [Authentication](/docs/intro#authentication) - OAuth2 setup
- [llama.cpp Documentation](https://github.com/ggerganov/llama.cpp) - GPU requirements and performance tuning
