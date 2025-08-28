# CLAUDE.md

This file provides guidance to Claude Code when working with the GitHub CI/CD infrastructure for BodhiApp.

## Purpose

The GitHub CI/CD system orchestrates automated build, test, and deployment workflows for the BodhiApp multi-crate workspace. It provides comprehensive automation for cross-platform builds, multi-variant Docker images, package publishing, and release management across Rust backend crates, Node.js bindings, TypeScript clients, and desktop applications.

## Key CI/CD Architecture

### Multi-Platform Build System
The CI/CD pipeline supports comprehensive cross-platform builds with matrix strategies covering macOS (ARM64), Linux (x86_64), and Windows (x86_64) platforms. The build system uses sophisticated caching strategies with Rust cache, npm cache, and Docker layer caching to optimize build times across all platforms.

### Automated Testing Infrastructure
The testing architecture integrates multiple test layers including Rust unit tests, integration tests, frontend React tests, NAPI binding tests, and end-to-end Playwright tests. All test suites run in parallel across platforms with comprehensive coverage reporting via Codecov integration.

### Multi-Variant Docker Publishing
The Docker publishing system builds and publishes multiple hardware-optimized variants (CPU, CUDA, ROCm, Vulkan) with multi-platform support for CPU images (AMD64 + ARM64). The system uses GitHub Container Registry (GHCR) with sophisticated tagging strategies for both production and development releases.

### Package Publishing Automation
The CI/CD system automates publishing for multiple package ecosystems including NPM packages for NAPI bindings (@bodhiapp/app-bindings), TypeScript client packages (@bodhiapp/ts-client), and desktop application releases with code signing and notarization for macOS.

### Release Management System
The release system supports multiple release types including desktop application releases with Tauri, Docker image releases with multi-variant support, and NPM package releases with automatic version bumping and post-release development version updates.

## Architecture Position

The GitHub CI/CD system serves as the central automation hub for the entire BodhiApp development and deployment lifecycle. It integrates with the workspace's multi-crate architecture by understanding dependency relationships and building crates in the correct order. The system coordinates with external services including GitHub Container Registry, NPM registry, Apple Developer services for code signing, and Codecov for coverage reporting.

## Cross-System Integration Patterns

### Workspace-Aware Build Orchestration
The CI/CD system understands the BodhiApp workspace structure and builds components in dependency order. It uses cargo metadata to dynamically discover workspace packages and applies targeted builds based on changed files, optimizing build times while ensuring comprehensive testing.

### Artifact Coordination System
The pipeline implements sophisticated artifact management where build outputs from one job become inputs for subsequent jobs. NAPI bindings built in the build phase are consumed by Playwright tests, llama-server binaries are shared across test phases, and Docker images coordinate with release processes.

### Multi-Registry Publishing Coordination
The system coordinates publishing across multiple registries (NPM, GHCR, GitHub Releases) with consistent versioning strategies. It implements atomic release processes where all components of a release succeed or fail together, maintaining system consistency.

### External Service Integration
The CI/CD system integrates with multiple external services including Apple Developer services for macOS code signing and notarization, Codecov for coverage reporting, and GitHub Container Registry for Docker image hosting. It manages authentication and credentials securely across all integrations.

## Important Constraints

### Security and Credential Management
All sensitive credentials are managed through GitHub Secrets with strict access controls. The system implements least-privilege access patterns and uses short-lived tokens where possible. Apple Developer credentials for code signing are handled with special security considerations including keychain management.

### Build Time and Resource Optimization
The CI/CD system implements aggressive caching strategies to minimize build times while maintaining reliability. It uses GitHub Actions cache for Rust builds, npm dependencies, and Docker layers. Build matrices are optimized to run in parallel while respecting GitHub Actions concurrency limits.

### Cross-Platform Compatibility Requirements
All workflows must handle platform-specific differences in shell commands, file paths, and binary formats. The system uses conditional logic to handle Windows PowerShell vs Unix bash differences and manages platform-specific dependencies and build tools.

### Release Consistency and Atomicity
Release processes implement consistency checks to ensure all components are released together with matching versions. The system includes rollback capabilities and prevents partial releases that could leave the system in an inconsistent state.

### Integration Test Environment Management
The CI/CD system manages complex integration test environments including authentication servers, test databases, and external service mocking. It coordinates test data management and ensures test isolation across parallel builds.