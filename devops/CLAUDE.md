# CLAUDE.md

See [PACKAGE.md](./PACKAGE.md) for implementation details and file references.

This file provides guidance to Claude Code when working with the devops deployment infrastructure.

## Purpose

The `devops` directory serves as BodhiApp's **containerized deployment foundation**, providing Docker build configurations for multiple hardware platforms (CPU, CUDA, ROCm, Vulkan) and comprehensive build automation that enables consistent deployment across development, staging, and production environments.

### Strategic Role in BodhiApp Ecosystem

This infrastructure bridges the gap between BodhiApp's multi-crate Rust workspace and production deployment environments. It abstracts hardware diversity (CPU-only, NVIDIA CUDA, AMD ROCm, cross-vendor Vulkan) while maintaining performance optimization for AI workload acceleration. The devops system enables BodhiApp to achieve **8-12x performance improvements** on GPU-accelerated deployments compared to unoptimized CPU execution.

## Key Domain Architecture

### Multi-Platform Docker Build System

BodhiApp's containerized deployment supports diverse hardware configurations through a **layered abstraction strategy**:

- **CPU Variants**: AMD64 and ARM64 architectures with llama.cpp base image optimization
  - AMD64: Full compilation from source with Rust workspace integration
  - ARM64: Binary download strategy with GitHub token authentication for faster builds

- **GPU Acceleration**: Hardware-specific optimization for AI inference acceleration
  - **NVIDIA CUDA**: Flash attention, full GPU offloading, KV cache quantization (8-12x speedup)
  - **AMD ROCm**: HIP compute library integration with row-split memory management
  - **Cross-vendor Vulkan**: Device enumeration and driver coordination for broad GPU support

- **Build Variants**: Environment-specific optimization with consistent dependency management
  - **Development**: Debug symbols, faster incremental builds, logging enhancement
  - **Production**: Release optimization, size minimization, security hardening

- **Base Image Strategy**: Specialized ghcr.io/bodhisearch/llama.cpp images eliminate compilation overhead
  - Pre-built llama-server executables with hardware-specific optimizations
  - Consistent runtime environment across all deployment variants

- **Multi-Stage Build Architecture**: Docker layer caching optimization for development velocity
  - **Dependency Stage**: Isolated cargo dependency compilation with ci_optims pre-compilation
  - **TypeScript Stage**: OpenAPI-driven client generation during container build
  - **Application Stage**: Rust workspace compilation with platform-specific targeting

### Hardware-Optimized Configuration System
Each Docker variant includes specialized llama.cpp server configurations:
- **CUDA Optimization**: Flash attention, full GPU offloading, KV cache quantization for 8-12x performance improvement
- **ROCm Integration**: AMD GPU acceleration with HIP-based compute and row-split memory management
- **Vulkan Compatibility**: Cross-vendor GPU acceleration for diverse hardware environments
- **CPU Tuning**: Thread optimization and memory mapping configuration for CPU-only deployments
- **Platform Detection**: Automatic target architecture selection (x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu)

### Build Automation and Orchestration
Comprehensive Makefile system coordinates local development and CI/CD workflows:
- **Delegated Build Targets**: Root Makefile delegates to devops/Makefile for centralized build logic
- **Development Workflow**: Quick iteration with development builds, local testing, and cleanup automation
- **Production Optimization**: Release mode compilation with size and performance optimizations
- **Multi-Platform Support**: Buildx integration for cross-platform image generation
- **Container Orchestration**: Volume mounting, environment configuration, and health check integration

### Deployment Strategy Architecture
Container deployment system designed for flexible hosting environments:
- **Environment Configuration**: YAML-based defaults with environment variable override capability
- **Data Persistence**: Structured volume mounting for model cache, application data, and configuration
- **Security Model**: Non-root container execution with proper file ownership and permission management
- **Health Monitoring**: HTTP-based health checks with GPU availability validation for accelerated variants
- **Resource Management**: Memory limits, GPU device access, and container lifecycle management

## Architecture Position

The devops infrastructure serves as BodhiApp's **deployment orchestration layer**:
- **Build Foundation**: Coordinates Rust workspace compilation, TypeScript client generation, and dependency management
- **Hardware Abstraction**: Provides consistent deployment interface across diverse hardware configurations
- **Development Bridge**: Enables seamless transition from local development to containerized deployment
- **CI/CD Integration**: Supports GitHub Actions workflows for automated building, testing, and publishing
- **Production Readiness**: Handles security, performance, and reliability requirements for production deployment

## Cross-Platform Integration Patterns

### Build System Coordination
The devops system integrates with BodhiApp's build infrastructure:
- **Workspace Integration**: Cargo workspace compilation with ci_optims dependency pre-compilation for Docker layer caching
- **TypeScript Client**: Automated ts-client generation from OpenAPI specifications during container build
- **Feature Flag Management**: Production and development feature selection with consistent optimization levels
- **Target Architecture**: Platform-specific compilation targeting (x86_64, aarch64) with llama_server_proc coordination
- **Dependency Optimization**: Multi-stage builds with dependency caching for faster iteration cycles

### Runtime Configuration Integration
Container runtime coordinates with BodhiApp's configuration system:
- **Settings Service**: YAML defaults integration with environment variable override capability
- **Path Management**: Structured directory layout for BODHI_HOME, HF_HOME, and executable lookup paths
- **Version Information**: Build-time version and commit SHA injection for runtime identification
- **Server Configuration**: Host binding, port exposure, and health check endpoint coordination
- **Logging Integration**: RUST_LOG environment variable with structured logging output

### Hardware Acceleration Integration
GPU-accelerated variants coordinate with llama.cpp server execution:
- **CUDA Integration**: Comprehensive optimization flags for NVIDIA GPU acceleration with flash attention and KV cache quantization
- **ROCm Coordination**: AMD GPU device access with HIP compute library integration
- **Vulkan Support**: Cross-vendor GPU acceleration with device enumeration and driver coordination
- **Fallback Strategy**: CPU execution capability maintained across all GPU variants for deployment flexibility
- **Performance Monitoring**: GPU utilization validation and memory usage optimization

### Development Workflow Integration
Local development workflow coordinates with containerized deployment:
- **Volume Mounting**: Local data persistence with docker-data directory structure
- **Port Forwarding**: Consistent localhost:8080 access across all deployment variants
- **Environment Parity**: Development and production configuration consistency
- **Debugging Support**: Debug symbol inclusion and logging configuration for development builds
- **Cleanup Automation**: Image management and cleanup targets for development iteration

## Important Constraints

### Build System Requirements
- **Docker Buildx**: Multi-platform build capability required for ARM64 and cross-platform support
- **Node.js Integration**: LTS version 22 required for TypeScript client generation during build
- **GitHub Token**: GH_PAT environment variable required for ARM64 binary downloads
- **Base Image Dependency**: Requires ghcr.io/bodhisearch/llama.cpp base images for hardware-specific optimization

### Hardware Platform Limitations
- **GPU Variants**: CUDA, ROCm, and Vulkan variants support AMD64 architecture only
- **ARM64 Support**: Limited to CPU variant with binary download strategy
- **Driver Requirements**: GPU variants require compatible host drivers (NVIDIA, AMD, Vulkan)
- **Memory Requirements**: GPU variants require sufficient VRAM (12-16GB recommended for 14B models)

### Security and Performance Constraints
- **Non-Root Execution**: All containers run as llama user with proper file ownership
- **Resource Limits**: Container memory and GPU device access must be properly configured
- **Health Check Requirements**: HTTP endpoint availability required for container orchestration
- **Data Persistence**: Volume mounting required for model cache and application data persistence

### Deployment Environment Requirements
- **Container Runtime**: Docker with GPU runtime support for accelerated variants
- **Network Configuration**: Port 8080 availability for HTTP server access
- **Storage Requirements**: Persistent volumes for model cache and application data
- **Environment Variables**: Proper configuration of BODHI_* environment variables for production deployment