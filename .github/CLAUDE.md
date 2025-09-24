# CLAUDE.md

See [PACKAGE.md](.github/PACKAGE.md) for implementation details and file references.

This file provides architectural guidance for the comprehensive GitHub CI/CD infrastructure that orchestrates the entire BodhiApp development, testing, and deployment lifecycle.

## Strategic Architecture Vision

The `.github` infrastructure implements a sophisticated multi-dimensional CI/CD orchestration system designed around BodhiApp's unique architectural challenges: cross-platform desktop deployment, multi-variant GPU acceleration, complex workspace dependencies, and diverse package ecosystem publishing. This system transcends simple automation by implementing intelligent build matrices, artifact coordination pipelines, and release consistency guarantees across heterogeneous technology stacks.

### Architectural Philosophy

The CI/CD design philosophy centers on three core principles:

1. **Workspace-Aware Intelligence**: The system understands BodhiApp's complex crate interdependencies and executes builds in optimal dependency order while enabling parallel execution where possible.

2. **Multi-Dimensional Scaling**: Rather than simple platform matrices, the system implements multi-variant builds (CPU/CUDA/ROCm/Vulkan) combined with multi-platform targets, creating a sophisticated build space that optimizes for both development velocity and deployment flexibility.

3. **Atomic Release Consistency**: All publishing operations are coordinated to ensure system-wide version consistency, preventing partial releases that could compromise the integrated application ecosystem.

## Core Architectural Systems

### Intelligent Build Orchestration Framework

The build system implements a sophisticated dependency-aware orchestration that goes beyond simple matrix builds. It uses dynamic cargo metadata analysis to determine optimal build ordering while maximizing parallelization opportunities. The system distinguishes between fast-feedback builds (Linux-only for rapid iteration) and comprehensive multi-platform builds (for releases), optimizing developer experience without compromising release quality.

**Key Innovation**: The build system maintains separate artifact streams for different consumption patterns - NAPI bindings flow to Playwright tests, llama-server binaries coordinate across test phases, and UI builds integrate with both NAPI and desktop packaging workflows.

### Multi-Layered Testing Architecture

The testing framework implements a hierarchical test execution strategy that coordinates five distinct testing domains:

1. **Rust Unit/Integration Layer**: Workspace-aware test execution with coverage aggregation
2. **Frontend Unit Testing**: React/TypeScript test suites with isolated coverage reporting  
3. **NAPI Binding Validation**: Cross-language interface testing with binary dependency coordination
4. **End-to-End Playwright Integration**: Full-stack behavior validation with authentication flow testing
5. **Cross-Platform Consistency Testing**: Ensuring identical behavior across target platforms

**Critical Design**: The system implements conditional test execution based on coverage success, preventing expensive E2E tests when core functionality is broken.

### Multi-Variant Container Orchestration

The Docker publishing system represents a unique architectural achievement in multi-variant container deployment. Rather than building separate images per variant, the system coordinates parallel builds across hardware acceleration types (CPU/CUDA/ROCm/Vulkan) while maintaining consistent base layers and optimized caching strategies.

**Advanced Features**:
- Multi-platform CPU images (AMD64 + ARM64) with automatic platform detection
- Hardware-specific optimization for GPU variants
- Intelligent tagging strategies supporting both versioned and latest deployments
- Development/production build variant coordination

### Cross-Ecosystem Package Publishing

The publishing system coordinates releases across fundamentally different package ecosystems while maintaining version consistency and atomic release semantics. This includes NPM package publishing (@bodhiapp/app-bindings, @bodhiapp/ts-client), GitHub Container Registry coordination, and desktop application distribution with platform-specific code signing.

## Strategic Integration Architecture

### Workspace Coordination Intelligence

The CI/CD system functions as the central nervous system for BodhiApp's distributed architecture, implementing deep workspace understanding that goes beyond simple dependency tracking. It maintains semantic understanding of crate roles (foundation, service, API, application) and coordinates build execution to optimize both development feedback loops and release consistency.

**Integration Patterns**:
- Dynamic workspace discovery using cargo metadata analysis
- Selective rebuild triggers based on changed file patterns and dependency graphs
- Coordinated artifact sharing between jobs with intelligent caching strategies
- Cross-platform binary coordination for complex integration testing scenarios

### External Service Ecosystem Integration

The system implements sophisticated integration patterns with multiple external service providers, each requiring different authentication, API, and consistency patterns:

**Authentication Services**: OAuth2 integration testing with Keycloak coordination, supporting both production (`id.getbodhi.app`) and development (`main-id.getbodhi.app`) authentication realms.

**Container Registries**: GitHub Container Registry integration with advanced tagging strategies, multi-platform manifests, and automated cleanup policies.

**Package Registries**: NPM publishing with scope-aware versioning (@bodhiapp namespace), automated post-release version bumping, and dependency consistency validation.

**Code Signing Services**: Apple Developer Program integration for macOS app notarization, keychain management, and certificate lifecycle coordination.

## Advanced Integration Patterns

### Dependency-Aware Build Orchestration

The system implements advanced build orchestration that combines workspace topology analysis with execution optimization. Rather than building all crates unconditionally, it performs intelligent change detection and builds only affected crates while ensuring dependency consistency.

**Pattern Implementation**:
- Cargo metadata parsing for dynamic workspace discovery
- File change pattern analysis for selective build triggering
- Cross-crate dependency validation ensuring build order correctness
- Parallel execution optimization within dependency constraint boundaries

### Artifact Flow Coordination Architecture

The pipeline implements a sophisticated artifact flow system where build products become precisely coordinated inputs for downstream processes. This coordination extends beyond simple file passing to include metadata propagation, version consistency validation, and dependency resolution.

**Critical Flows**:
- NAPI bindings → Playwright test execution (with binary verification)
- llama-server binaries → Integration test coordination (with platform-specific path resolution)
- UI builds → Desktop application packaging (with embedded resource optimization)
- Coverage reports → Aggregated reporting (with multi-language consolidation)

### Multi-Registry Atomic Publishing

The publishing system implements true atomic semantics across heterogeneous registry types, ensuring that partial publications cannot occur even in the presence of network failures or service outages.

**Consistency Guarantees**:
- Version propagation validation across NPM packages and Docker images
- Release artifact coordination ensuring matching SHA commitments
- Rollback capability implementation for failed multi-registry deployments
- Post-release validation ensuring all published artifacts are accessible

### Secure Credential Orchestration

The system manages complex credential flows across multiple external services while maintaining security boundaries and implementing least-privilege access patterns.

**Security Architecture**:
- GitHub Secrets integration with role-based access control
- Apple Developer credential lifecycle management (certificates, provisioning profiles, app-specific passwords)
- NPM token management with scope-limited publishing permissions
- Container registry authentication with temporary token generation

## Critical Architectural Constraints

### Security Architecture and Threat Model

The CI/CD system operates under a comprehensive security model that recognizes the diverse attack surfaces present in multi-platform, multi-registry deployment scenarios. Security implementation goes beyond basic credential management to include supply chain integrity, artifact authenticity validation, and secure build environment isolation.

**Security Implementation**:
- Multi-layer credential isolation using GitHub Secrets with time-bounded access
- Apple Developer credential security including keychain isolation and certificate validation
- Supply chain security through dependency pinning and submodule verification
- Build environment isolation preventing cross-contamination between matrix builds
- Artifact integrity verification using cryptographic signatures and checksums

### Resource Optimization and Concurrency Management

The system implements sophisticated resource optimization that balances build speed, GitHub Actions quota consumption, and parallel execution efficiency. This includes intelligent caching hierarchies, build matrix optimization, and resource contention prevention.

**Optimization Strategies**:
- Multi-tier caching architecture (Rust compilation cache, npm cache, Docker layer cache)
- Build matrix optimization reducing redundant work while maintaining platform coverage
- Concurrency group management preventing resource conflicts during parallel execution
- Selective execution based on change detection to minimize unnecessary resource consumption

### Cross-Platform Consistency Requirements

The system must maintain functional and behavioral consistency across fundamentally different operating systems while accommodating platform-specific requirements and constraints.

**Consistency Challenges**:
- Shell environment differences (bash vs PowerShell vs cmd)
- File path handling (Unix vs Windows path separators)
- Binary format compatibility and cross-compilation requirements
- Platform-specific dependency management (apt vs homebrew vs chocolatey)
- Code signing and notarization requirements varying by platform

### Release Atomicity and Consistency Guarantees

The release system implements distributed transaction semantics across multiple external services, ensuring that releases maintain system-wide consistency even in the presence of partial failures.

**Consistency Implementation**:
- Pre-flight validation ensuring all prerequisites are satisfied before beginning release processes
- Atomic version propagation across all package ecosystems with validation checkpoints
- Comprehensive rollback procedures for handling partial release failures
- Post-release validation ensuring all published artifacts are accessible and functional
- Dependency consistency validation preventing version mismatch scenarios

### Integration Test Environment Complexity

The system coordinates complex integration test environments that must replicate production-like conditions while maintaining test isolation and deterministic behavior across parallel execution contexts.

**Environment Management**:
- Multi-service orchestration including authentication servers, external API mocking, and database coordination
- Test data lifecycle management ensuring isolation between concurrent test executions
- Network isolation preventing cross-test contamination while enabling external service access
- Resource cleanup ensuring integration tests don't leak resources or affect subsequent executions
- Authentication flow testing requiring coordination with external OAuth2 providers in test mode

## Extension and Maintenance Patterns

### Adding New Build Variants

When adding new hardware acceleration variants (e.g., Intel Arc, Apple Metal), the system architecture supports extension through the matrix build pattern in `.github/workflows/publish-docker.yml`. New variants require coordinated updates across Dockerfiles, build matrices, and documentation generation.

**Extension Points**:
- Matrix variant definition with platform-specific constraints
- Dockerfile creation following naming conventions (`devops/{variant}.Dockerfile`)
- Cache scope coordination to prevent cross-variant cache pollution
- Documentation template updates for new hardware requirements

### Platform Support Expansion

Adding new target platforms requires coordinated updates across multiple workflow files and actions, with particular attention to cross-platform consistency requirements and artifact coordination.

**Integration Requirements**:
- Platform-specific setup actions (following `setup-{platform}` naming pattern)
- Build matrix updates with appropriate target triple specifications
- Artifact path coordination ensuring consistent file locations
- Test execution adaptation for platform-specific requirements

### Monitoring and Observability Integration

The system architecture supports integration with external monitoring and observability platforms through structured output, metric emission, and failure notification patterns.

**Observability Extensions**:
- Structured logging with consistent formatting across all workflow steps
- Metric emission for build times, test coverage, and deployment success rates
- Integration points for external monitoring systems (Datadog, Prometheus, etc.)
- Failure notification coordination with team communication platforms

## Domain-Specific Implementation Guidance

### Workflow Trigger Strategy

The system implements sophisticated trigger strategies that balance responsive development feedback with resource conservation. Push events to main/working branches trigger fast Linux builds, while pull requests trigger comprehensive validation suites.

**Trigger Design Rationale**:
- Path-based filtering prevents unnecessary builds when only documentation changes
- Manual workflow dispatch enables controlled testing of complex scenarios
- Tag-based triggers coordinate release processes with semantic versioning
- Concurrency group management prevents conflicting simultaneous builds

### Build Matrix Optimization Philosophy

Rather than exhaustive cross-platform testing for every change, the system implements tiered validation where fast feedback loops use Ubuntu-only builds, while comprehensive platform validation occurs during release processes.

**Matrix Strategy**:
- Development builds prioritize speed (Linux-only with comprehensive test suite)
- Release builds prioritize coverage (multi-platform with reduced parallelization)
- Docker builds implement variant-specific optimization (CPU multi-platform, GPU single-platform)
- Artifact coordination ensures platform-specific binaries reach appropriate test environments

### Dependency Management Strategy

The system implements strict dependency management patterns that ensure reproducible builds while enabling security updates and feature integration.

**Dependency Patterns**:
- Submodule management with authenticated access using GitHub PAT
- Python dependency pinning with specific requirements.txt files
- Node.js dependency management with package-lock.json consistency
- Rust dependency caching with cargo registry and git dependency coordination