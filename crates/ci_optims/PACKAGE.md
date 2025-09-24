# PACKAGE.md - ci_optims Crate

Implementation index and navigation guide for the CI/CD optimization crate.

## Overview

The `ci_optims` crate is a build-time optimization dummy crate that pre-compiles all heavy workspace dependencies in Docker builds, reducing CI build times from 45+ minutes to 5-10 minutes through strategic layer caching.

## Core Implementation Files

### Dependency Aggregation

**crates/ci_optims/Cargo.toml**
```toml
  [dependencies]
  # Include all heavy dependencies from workspace to pre-compile them
  aes-gcm = { workspace = true }
  anyhow = { workspace = true }
  # ... 80+ workspace dependencies
```
Complete dependency registry covering all functional domains: security, networking, async runtime, serialization, AI/ML, and observability.

**crates/ci_optims/src/lib.rs**
```rust
  // Dummy crate for pre-compiling dependencies in CI builds
  #![allow(unused_imports)]
  #![allow(clippy::single_component_path_imports)]
  
  // Import heavy dependencies to trigger their compilation
  use aes_gcm;
  use async_openai;
  use axum;
  // ... strategic unused imports
  
  pub fn dummy_function() {
    println!("CI optimizations crate loaded");
  }
```
Minimal implementation with unused imports that trigger dependency compilation during build without runtime overhead.

## Build Integration Infrastructure

### Workspace Filtering System

**scripts/filter-cargo-toml.py**
```python
  def filter_cargo_toml(input_file, output_file):
    """Filter Cargo.toml to create minimal workspace for dependency compilation."""
    # Remove local workspace dependencies to avoid missing crate errors
    # Keep only ci_optims crate in members section
    # Generate filtered lock file for isolated dependency resolution
```
Python script that creates minimal workspace configuration for dependency-only compilation stage.

### Docker Build Integration

**devops/cpu.Dockerfile**
```dockerfile
  COPY crates/ci_optims/ crates/ci_optims/
  RUN python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml
  RUN if [ "$BUILD_VARIANT" = "production" ]; then \
        cargo build --release -p ci_optims; \
      else \
        cargo build -p ci_optims; \
      fi
```
Multi-stage Docker build pattern replicated across CPU, CUDA, ROCm, and Vulkan variants for consistent dependency pre-compilation.

## Build Commands & Operations

### Development Commands

**Local dependency testing:**
```bash
# Build ci_optims to test dependency compilation
cargo build -p ci_optims

# Release mode compilation (matches production builds)
cargo build --release -p ci_optims
```

**Docker build integration:**
```bash
# Create filtered workspace for dependency-only build
python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml

# Build with filtered workspace
cargo build -p ci_optims --manifest-path Cargo.filtered.toml
```

### CI/CD Integration

**Multi-platform builds:**
```bash
# CPU variant build
docker build -f devops/cpu.Dockerfile --build-arg BUILD_VARIANT=production .

# GPU variant builds
docker build -f devops/cuda.Dockerfile .
docker build -f devops/rocm.Dockerfile .
docker build -f devops/vulkan.Dockerfile .
```

## Dependency Categories and Usage

### Security & Cryptography Dependencies
- **aes-gcm, pbkdf2, rsa, sha2**: Core cryptographic operations
- **oauth2, jsonwebtoken**: Authentication flows
- **keyring**: Secure credential management

### Web Services Stack
- **axum, tower, hyper**: HTTP server infrastructure
- **reqwest**: HTTP client operations  
- **tower-sessions, cookie**: Session management

### Async Runtime Foundation  
- **tokio, futures**: Asynchronous operation foundation
- **async-trait**: Trait abstraction for async code

### Data Processing
- **serde family**: Universal serialization
- **sqlx**: Database operations and query compilation
- **validator**: Input validation

### AI/ML Integration
- **async-openai**: OpenAI API integration
- **hf-hub**: Hugging Face model management

### Observability
- **tracing ecosystem**: Structured logging and instrumentation
- **anyhow**: Error context and propagation

## Maintenance and Synchronization

### Dependency Audit Workflow
```bash
# Compare ci_optims dependencies with actual workspace usage
cargo tree -p ci_optims --format "{p} {f}"
cargo tree --workspace --format "{p} {f}" | sort | uniq
```

### Build Variant Testing
```bash
# Test all build variants for consistency
docker build -f devops/cpu.Dockerfile --target deps-build .
docker build -f devops/cuda.Dockerfile --target deps-build .
docker build -f devops/rocm.Dockerfile --target deps-build .
docker build -f devops/vulkan.Dockerfile --target deps-build .
```

### Performance Monitoring
```bash
# Monitor Docker layer cache effectiveness
docker history <image_id> --no-trunc
docker system df --verbose
```

## Important Notes

- **Build-time only**: Never included in runtime applications
- **Manual synchronization**: Requires updates when workspace dependencies change
- **Platform agnostic**: Contains only platform-independent dependencies
- **Memory intensive**: Dependency compilation consumes significant build resources
- **Cache sensitive**: Docker layer invalidation affects build performance significantly