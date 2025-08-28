# CLAUDE.md

This file provides guidance to Claude Code when working with the `ci_optims` crate.

## Purpose

The `ci_optims` crate serves as BodhiApp's **CI/CD optimization dummy crate**, designed specifically to enable Docker layer caching and build acceleration by pre-compiling all heavy workspace dependencies in a separate, cacheable Docker layer before the main application build.

## Key Domain Architecture

### Dependency Pre-Compilation System
BodhiApp's build optimization strategy uses ci_optims as a compilation trigger:
- **Comprehensive dependency inclusion**: Contains all 80+ heavy workspace dependencies from the root Cargo.toml workspace.dependencies section
- **Dummy implementation pattern**: Minimal lib.rs with unused imports that trigger full dependency compilation without actual functionality
- **Docker layer separation**: Enables Docker to cache the expensive dependency compilation step independently from application source changes
- **Build variant consistency**: Supports both debug and release mode compilation matching the target application build configuration

### CI/CD Build Pipeline Integration
Sophisticated multi-stage Docker build process leveraging ci_optims for optimization:
- **Filtered workspace creation**: Python script (scripts/filter-cargo-toml.py) creates minimal workspace containing only ci_optims
- **Dependency-only build stage**: First Docker stage compiles ci_optims dependencies in isolation, creating a cacheable layer
- **Source code isolation**: Application source changes don't invalidate the expensive dependency compilation cache
- **Multi-platform support**: Works across CPU, CUDA, ROCm, and Vulkan build variants with consistent caching behavior
- **Build time reduction**: Reduces CI build times from ~45 minutes to ~5-10 minutes on cache hits

### Docker Layer Caching Architecture
Strategic Docker layer organization for maximum cache efficiency:
- **Layer 1**: System dependencies and Rust toolchain (rarely changes)
- **Layer 2**: Workspace configuration and ci_optims dependency compilation (changes only when dependencies update)
- **Layer 3**: TypeScript client build (changes when OpenAPI spec updates)
- **Layer 4**: Application source and final binary compilation (changes with every code update)
- **Cache invalidation strategy**: Only Layer 4 rebuilds on typical development changes, preserving expensive compilation work

### Workspace Dependency Coordination
Central registry of all heavy dependencies used across BodhiApp crates:
- **Cryptographic libraries**: aes-gcm, pbkdf2, rsa, sha2 for security services
- **HTTP/networking stack**: axum, hyper, reqwest, tower ecosystem for web services
- **Async runtime**: tokio, futures ecosystem for concurrent operations
- **Serialization**: serde, serde_json, serde_yaml for data interchange
- **Database integration**: sqlx for persistence layer
- **AI/ML libraries**: async-openai, hf-hub for model management
- **Authentication**: oauth2, jsonwebtoken for security
- **Observability**: tracing, tracing-subscriber for logging and monitoring

## Architecture Position

The ci_optims crate occupies a unique position in BodhiApp's build architecture:
- **Build-time only**: Never used at runtime, exists solely for compilation optimization
- **Workspace dependency aggregator**: Centralizes all heavy dependencies to enable efficient pre-compilation
- **CI/CD enabler**: Critical component of the Docker build strategy that makes multi-platform builds feasible
- **Development acceleration**: Significantly improves developer experience by reducing build times in containerized environments
- **Deployment optimization**: Enables faster CI/CD pipelines and more efficient container image builds

## Important Constraints

### Build System Requirements
- **Docker-specific optimization**: Only provides benefits in containerized build environments with layer caching
- **Workspace dependency synchronization**: Must be manually updated when new heavy dependencies are added to other crates
- **Build variant consistency**: Must be compiled with the same optimization level (debug/release) as the target application
- **Platform independence**: Contains only platform-agnostic dependencies, platform-specific optimizations handled elsewhere

### Maintenance Considerations
- **Dependency drift prevention**: Requires periodic synchronization with actual workspace dependencies
- **Build script coordination**: Changes must be coordinated with scripts/filter-cargo-toml.py and Docker build stages
- **Cache invalidation awareness**: Understanding of when Docker layer cache becomes invalid is critical for effective use
- **Resource usage**: Pre-compilation consumes significant CPU and memory resources during the dependency build stage