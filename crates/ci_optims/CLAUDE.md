# CLAUDE.md

See [PACKAGE.md](./PACKAGE.md) for implementation details and file references.

This file provides guidance to Claude Code when working with the `ci_optims` crate.

## Purpose

The `ci_optims` crate serves as BodhiApp's **CI/CD optimization dummy crate**, designed specifically to enable Docker layer caching and build acceleration by pre-compiling all heavy workspace dependencies in a separate, cacheable Docker layer before the main application build.

This crate implements a sophisticated dependency pre-compilation strategy that reduces CI build times from ~45 minutes to ~5-10 minutes on cache hits across CPU, CUDA, ROCm, and Vulkan build variants.

## Key Domain Architecture

### Dependency Pre-Compilation Strategy

BodhiApp's build optimization leverages Docker's layer caching mechanism through ci_optims:

- **Comprehensive dependency aggregation**: Contains all 80+ heavy workspace dependencies, ensuring complete compilation coverage
- **Dummy implementation pattern**: Minimal lib.rs with strategic unused imports that trigger full dependency compilation without runtime overhead
- **Docker layer isolation**: Creates a separate, cacheable compilation layer that survives application source code changes
- **Multi-variant support**: Consistent compilation across debug/release modes and CPU/GPU platform variants
- **Workspace filtering integration**: Works with Python filtering scripts to create minimal compilation workspaces

### Multi-Stage Docker Build Architecture

Sophisticated Docker build pipeline optimized for dependency caching:

**Stage 1: Filtered Workspace Preparation**
- Python script creates minimal Cargo.toml containing only ci_optims crate
- Removes all local workspace dependencies to avoid missing crate errors
- Generates filtered lock file for isolated dependency resolution

**Stage 2: Dependency Pre-Compilation**
- Compiles ci_optims with all heavy dependencies in isolation
- Creates cacheable Docker layer that persists across source code changes
- Supports conditional debug/release compilation based on BUILD_VARIANT

**Stage 3: TypeScript Client Build**
- Independent layer for OpenAPI-generated TypeScript client
- Cached separately from Rust dependencies and application source

**Stage 4: Application Compilation**
- Final stage that rebuilds only on source code changes
- Benefits from pre-compiled dependencies in earlier cached layers

### Cache Invalidation and Layer Strategy

Strategically designed Docker layers for optimal cache hit ratios:

**Layer Hierarchy:**
1. **Base System Layer**: Rust toolchain, system dependencies (stable)
2. **Dependency Compilation Layer**: ci_optims build output (invalidated only on dependency changes)
3. **TypeScript Client Layer**: Generated API client (invalidated on OpenAPI spec changes)
4. **Application Layer**: Final binary compilation (invalidated on source changes)

**Cache Efficiency Patterns:**
- Typical development changes only invalidate Layer 4
- Dependency updates invalidate Layers 2-4
- API changes invalidate Layers 3-4
- System updates invalidate all layers

This hierarchy ensures that expensive dependency compilation work is preserved across the majority of development iterations.

### Dependency Domain Coverage

Central registry categorized by functional domains:

**Security & Cryptography:**
- aes-gcm, pbkdf2, rsa, sha2: Core cryptographic operations
- oauth2, jsonwebtoken: Authentication and authorization
- keyring: Secure credential storage

**Web Services & Networking:**
- axum, tower, hyper: HTTP server infrastructure
- reqwest: HTTP client operations
- tower-sessions, cookie: Session management

**Async Runtime & Concurrency:**
- tokio, futures ecosystem: Asynchronous operation foundation
- async-trait: Trait abstraction for async operations

**Data Processing & Serialization:**
- serde family: Universal data interchange
- sqlx: Database abstraction and query compilation
- validator: Input validation and sanitization

**AI/ML Integration:**
- async-openai: OpenAI API client
- hf-hub: Hugging Face model repository integration

**Observability & Debugging:**
- tracing ecosystem: Structured logging and instrumentation
- anyhow: Error handling and context propagation

## Strategic Architecture Position

The ci_optims crate serves a critical but invisible role in BodhiApp's build ecosystem:

**Build Infrastructure Role:**
- **Compilation catalyst**: Triggers dependency compilation without runtime footprint
- **Cache layer coordinator**: Orchestrates Docker layer caching strategy
- **Multi-platform enabler**: Supports consistent builds across CPU/GPU variants
- **CI/CD accelerator**: Transforms infeasible 45-minute builds into practical 5-10 minute iterations

**Developer Experience Impact:**
- **Containerized development**: Makes Docker-based development workflows practical
- **CI/CD reliability**: Reduces build failures from timeout issues
- **Resource efficiency**: Minimizes compute costs in CI environments
- **Cross-platform consistency**: Ensures identical dependency compilation across build variants

## Important Constraints

### Build System Integration Requirements

**Environment Dependencies:**
- Docker with layer caching support (BuildKit recommended)
- Python 3.x for workspace filtering scripts
- Consistent Rust toolchain across build stages
- Sufficient build resources for parallel dependency compilation

**Synchronization Requirements:**
- Manual dependency updates when workspace dependencies change
- Coordination with filter-cargo-toml.py script modifications
- Alignment with Dockerfile build stage definitions
- Consistency across CPU/CUDA/ROCm/Vulkan build variants

### Operational Constraints and Maintenance

**Performance Trade-offs:**
- High initial compilation cost for cache population
- Memory-intensive dependency compilation phase
- Cache invalidation cascades on dependency updates
- Platform-specific compilation overhead for GPU variants

**Maintenance Workflows:**
- **Dependency auditing**: Periodic review of included dependencies vs. actual usage
- **Cache optimization**: Monitoring cache hit rates and invalidation patterns
- **Build variant testing**: Ensuring consistent behavior across all supported platforms
- **Resource scaling**: Adjusting build resources based on dependency compilation requirements

**Critical Failure Modes:**
- Dependency version mismatches between filter script and actual workspace
- Docker layer corruption requiring full cache invalidation
- Platform-specific compilation failures in multi-variant builds
- Resource exhaustion during heavy dependency compilation phases