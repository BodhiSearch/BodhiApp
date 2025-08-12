# GitHub Workflows Context - BodhiApp CI/CD System

## Overview

This document provides comprehensive context about the BodhiApp GitHub Actions workflow system, covering conventions, patterns, and architectural decisions for AI coding assistants working with CI/CD configurations.

## Workflow Architecture & Design Philosophy

### Core Principles

1. **Makefile-First Approach**: Complex shell logic is abstracted into Makefile targets rather than embedded in workflow YAML files
2. **Reusable Actions**: Common setup steps are extracted into reusable composite actions in `.github/actions/`
3. **Platform-Specific Optimizations**: Fast Linux-only workflows for quick feedback, comprehensive multiplatform workflows for release quality
4. **Artifact Management**: Binaries and test results are systematically uploaded/downloaded between jobs
5. **Fail-Fast vs. Comprehensive**: Different strategies for different workflow purposes

### Script Hierarchy Pattern

When implementing complex logic in workflows, follow this hierarchy:
1. **Simple commands**: Inline in workflow YAML
2. **Moderate complexity**: Makefile targets (preferred)
3. **Complex logic**: Python scripts in `scripts/` folder, invoked from Makefile
4. **Platform-specific**: PowerShell scripts in `scripts/` for Windows-specific operations

## Current Workflow Inventory

### Build & Test Workflows

#### `build.yml` - Fast Linux Build (Primary CI)
- **Purpose**: Quick feedback on every push/PR for rapid development
- **Platform**: Ubuntu Linux only (cost-effective, fast)
- **Triggers**: Push to `main`/`working` branches, PRs to `main`
- **Timeout**: 30 minutes (build), 15 minutes (Playwright)
- **Key Features**:
  - Uses all reusable actions for consistency
  - Full test suite including NAPI bindings and Playwright tests
  - Codecov coverage reporting
  - TypeScript client validation

#### `build-multiplatform.yml` - Comprehensive Multiplatform Build
- **Purpose**: Cross-platform validation for releases
- **Platforms**: macOS (aarch64), Linux (x86_64), Windows (x86_64)
- **Triggers**: Push to `main` branch only
- **Timeout**: 40 minutes (build), 20 minutes (Playwright)
- **Key Features**:
  - Matrix strategy across all supported platforms
  - Uses reusable actions for consistency
  - Comprehensive artifact management
  - Platform-specific binary handling

### Release Workflows

#### `release.yml` - Application Release
- **Purpose**: Build and release desktop application binaries
- **Triggers**: Git tags matching `v*` pattern, manual dispatch
- **Platforms**: Currently macOS only (commented out Linux/Windows)
- **Key Features**:
  - Tauri application building
  - DMG artifact generation
  - Draft/prerelease options via manual dispatch

#### `publish-ts-client.yml` - TypeScript Client Release
- **Purpose**: Publish `@bodhiapp/ts-client` npm package
- **Triggers**: Git tags matching `ts-client/v*` pattern
- **Platform**: Ubuntu Linux only
- **Key Features**:
  - Automatic version extraction from git tags
  - NPM package publishing with provenance
  - Post-release version bumping to `-dev`

#### `publish-npm-napi.yml` - NAPI Bindings Release
- **Purpose**: Publish `@bodhiapp/app-bindings` npm package with native bindings
- **Triggers**: Git tags matching `bodhi-app-bindings/v*` pattern
- **Platforms**: macOS (aarch64), Linux (x86_64), Windows (x86_64)
- **Key Features**:
  - Cross-platform NAPI binary compilation
  - Binary artifact collection and npm package assembly
  - Native module publishing

## Reusable Actions Architecture

### Action Design Patterns

All reusable actions follow consistent patterns:
- **Composite actions**: Use `using: composite` for shell-based actions
- **Platform abstraction**: Accept `platform` input for cross-platform logic
- **Conditional execution**: Platform-specific steps using `if: inputs.platform == 'condition'`
- **Shell consistency**: Use `shell: bash` for cross-platform compatibility where possible

### Current Reusable Actions

#### `.github/actions/setup-environment/`
- **Purpose**: Common environment setup (git config, submodules, Python packages)
- **Inputs**: `platform`, `gh-pat`
- **Key Operations**:
  - Git symlinks and line ending configuration
  - Submodule checkout with PAT authentication
  - Python package installation for test dependencies

#### `.github/actions/setup-models/`
- **Purpose**: HuggingFace model caching and download
- **Inputs**: `platform`
- **Key Features**:
  - Cross-platform cache with `enableCrossOsArchive: true`
  - Specific model version pinning for reproducibility
  - Cache key: `hf-cache-phi4-mini-instruct`

#### `.github/actions/setup-rust/`
- **Purpose**: Rust toolchain and component installation
- **Inputs**: `platform`, `target`
- **Key Features**:
  - Rust 1.87.0 with specific components (rustfmt, clippy, llvm-tools)
  - Rust cache via `Swatinem/rust-cache@v2`
  - Platform-specific Tauri CLI installation
  - Deranged dependency update workaround

#### `.github/actions/setup-node/`
- **Purpose**: Node.js and npm setup with caching
- **Inputs**: `platform`
- **Key Features**:
  - Node LTS (jod) version
  - NPM cache management across multiple package.json files
  - Platform-specific cache directory detection

#### `.github/actions/setup-playwright/`
- **Purpose**: Playwright browser installation and caching
- **Inputs**: `platform`, `working-directory`
- **Key Features**:
  - Playwright version detection from package.json
  - Cross-platform browser cache
  - Ubuntu 24.04 compatibility packages
  - Chromium-only installation for CI efficiency

#### `.github/actions/build-and-test/`
- **Purpose**: Core build and test execution with coverage
- **Inputs**: `platform`, `target`, various secret/config inputs
- **Outputs**: `coverage-success`
- **Key Operations**:
  - Makefile target execution (`ci.clean`, `ci.coverage`, `ci.ui`)
  - Codecov coverage reporting (backend and frontend)
  - Artifact uploads (llama_server_proc binaries)

#### `.github/actions/ts-client-check/`
- **Purpose**: TypeScript client validation and testing
- **Inputs**: `platform`
- **Key Operations**:
  - OpenAPI spec synchronization check
  - TypeScript client test execution

#### `.github/actions/napi-build/`
- **Purpose**: NAPI bindings compilation and artifact upload
- **Inputs**: `platform`, `target`
- **Key Features**:
  - Debug build for faster CI execution
  - Artifact upload with platform-specific naming

## Environment Variables & Secrets

### Standard Environment Variables
- `CI=true` - Enables CI mode across tools
- `RUST_BACKTRACE=1` - Enhanced Rust error reporting
- `BODHI_EXEC_VARIANT=cpu` - Specifies CPU variant for execution
- `CI_DEFAULT_VARIANT=cpu` - Default variant for CI builds

### Required Secrets
- `GH_PAT` - GitHub Personal Access Token for submodule access
- `CODECOV_TOKEN` - Codecov integration token
- `HF_TEST_TOKEN_ALLOWED` / `HF_TEST_TOKEN_PUBLIC` - HuggingFace test tokens
- Integration test credentials (various `INTEG_TEST_*` secrets)
- NPM publishing tokens for package releases

### Configuration Variables
- `INTEG_TEST_AUTH_URL` / `INTEG_TEST_AUTH_REALM` - Integration test configuration

## Artifact Management Patterns

### Naming Conventions
- Binary artifacts: `llama-server-binaries-{target}`
- NAPI bindings: `napi-bindings-{target}`
- Test results: `{test-type}-results-{platform}`
- Reports: `{report-type}-report-{platform}`

### Retention Policies
- Build artifacts: 1 day (quick CI feedback)
- Test results: 7 days (debugging failed tests)
- Release artifacts: Managed by GitHub releases

### Cross-Job Dependencies
- Build job uploads artifacts → Playwright job downloads artifacts
- Platform-specific artifact naming enables parallel execution
- Conditional job execution based on build success

## Platform-Specific Considerations

### Linux (Ubuntu)
- **Runner**: `ubuntu-latest-4-cores` for build jobs, `ubuntu-latest` for others
- **Dependencies**: Extensive apt package installation for Tauri/WebKit
- **Advantages**: Cost-effective, fast, comprehensive tooling support

### macOS
- **Runner**: `macos-latest` (aarch64-apple-darwin)
- **Considerations**: Higher cost, required for native macOS builds
- **Use Cases**: Release builds, cross-platform validation

### Windows
- **Runner**: `windows-latest`
- **Shell Preference**: PowerShell for Windows-specific operations, bash for cross-platform
- **Considerations**: Path separators, executable extensions (.exe)

## Testing Integration

### Test Execution Flow
1. **Backend Tests**: Rust unit and integration tests via `make ci.coverage`
2. **Frontend Tests**: React component tests via `make ci.ui`
3. **NAPI Tests**: Native binding tests via Vitest
4. **Playwright Tests**: End-to-end browser automation

### Test Reporting
- **JUnit XML**: Generated for GitHub test reporting integration
- **Coverage Reports**: Codecov integration for both backend (lcov) and frontend (JSON/XML)
- **Test Results**: `dorny/test-reporter@v1` for PR integration

## Makefile Integration

### Key CI Targets
- `ci.clean` - Clean all cargo packages
- `ci.coverage` - Generate code coverage with llvm-cov
- `ci.ui` - Run frontend tests with coverage
- `ci.ts-client-check` - Validate TypeScript client synchronization
- `ci.ts-client-test` - Execute TypeScript client tests

### Build Targets
- `ci.build` - Tauri application build
- `ci.app-npm` - Install npm dependencies

## Workflow Triggers & Branching Strategy

### Branch-Based Triggers
- **`main` branch**: Full multiplatform builds, release preparation
- **`working` branch**: Fast Linux builds for development
- **Pull requests**: Fast Linux builds for validation

### Tag-Based Triggers
- `v*` - Application releases
- `ts-client/v*` - TypeScript client releases  
- `bodhi-app-bindings/v*` - NAPI bindings releases

### Manual Triggers
- `workflow_dispatch` available on most workflows for manual execution
- Release workflows support draft/prerelease options

## Performance Optimizations

### Caching Strategy
- **Rust**: `Swatinem/rust-cache@v2` for cargo dependencies
- **Node**: NPM cache based on package-lock.json hashes
- **HuggingFace**: Model cache with cross-OS archive support
- **Playwright**: Browser cache with version-specific keys

### Parallel Execution
- Matrix strategies for cross-platform builds
- Separate jobs for build and test phases
- Artifact-based communication between jobs

### Resource Management
- Timeout configurations prevent runaway jobs
- `continue-on-error` for non-critical steps
- `fail-fast: false` for comprehensive platform testing

## Error Handling & Debugging

### Failure Modes
- **Build failures**: Rust compilation errors, dependency issues
- **Test failures**: Unit test failures, integration test timeouts
- **Artifact issues**: Missing binaries, upload/download failures

### Debugging Features
- Artifact uploads for failed test runs
- Verbose test output in CI logs
- Binary verification steps with detailed logging

### Recovery Strategies
- `continue-on-error` for coverage uploads
- Retry mechanisms via GitHub Actions (implicit)
- Manual workflow dispatch for recovery runs

## Conventions & Best Practices

### YAML Formatting
- 2-space indentation (consistent with project standards)
- Descriptive step names with action verbs
- Consistent input parameter naming across actions

### Security Practices
- Minimal secret exposure in logs
- PAT token usage for submodule access
- Provenance attestation for npm packages

### Maintenance Guidelines
- Reusable actions for consistency
- Version pinning for external actions
- Regular dependency updates via dependabot

## Future Considerations

### Scalability
- Action marketplace publishing for reusable components
- Workflow templates for new service additions
- Enhanced parallel execution strategies

### Monitoring
- Workflow execution time tracking
- Cost optimization for runner usage
- Success rate monitoring and alerting

### Integration
- Enhanced integration with external services
- Automated dependency updates
- Performance benchmarking integration

---

This context document reflects the current state of the BodhiApp GitHub Actions system as of the workflow restructuring that introduced reusable actions and the fast Linux-only build workflow for improved CI efficiency. 