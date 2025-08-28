# CLAUDE.md

This file provides guidance to Claude Code when working with the llama_server_proc crate.

## Purpose

The `llama_server_proc` crate provides comprehensive process management and HTTP client functionality for interacting with llama.cpp server processes in BodhiApp's local LLM inference infrastructure. It serves as the foundational layer for managing external llama.cpp server processes with sophisticated lifecycle management, health monitoring, and cross-platform binary distribution.

## Key Domain Architecture

### Process Management System

The crate implements a sophisticated process lifecycle management system that handles llama.cpp server processes with comprehensive monitoring and health checking. The architecture provides async trait-based abstraction (`Server` trait) enabling dependency injection and testing through mockall integration, while the concrete `LlamaServer` implementation manages actual process spawning, monitoring, and cleanup with automatic resource management through Drop trait implementation.

### Cross-Platform Binary Distribution Architecture

A comprehensive build system architecture handles platform-specific llama-server executable management through automated GitHub release downloading and local build orchestration. The system supports multiple acceleration variants (CPU, Metal, CUDA) with platform-specific executable naming and file locking for concurrent build safety, enabling seamless deployment across macOS (aarch64), Linux (x86_64), and Windows (x86_64) platforms.

### HTTP Proxy and Health Monitoring System

The architecture implements a sophisticated HTTP client system with connection pooling, TCP optimizations, and localhost security binding for proxying requests to llama.cpp server endpoints. Health monitoring uses exponential backoff polling with 300-second timeout and structured logging integration for process output monitoring, ensuring reliable server readiness detection and operational visibility.

### Configuration Builder Architecture

A builder pattern-based configuration system (`LlamaServerArgs`) provides flexible server parameter management with automatic port selection, optional API key authentication, and extensible server argument handling. The configuration system integrates with the objs crate's builder error handling and supports both programmatic construction and command-line argument serialization.

## Architecture Position

The `llama_server_proc` crate occupies a foundational infrastructure position in BodhiApp's architecture, serving as the lowest-level abstraction for external process management. It sits below the services layer and provides essential process management capabilities that higher-level services depend on for local LLM inference operations.

### Dependency Architecture

- **Foundation Layer**: Depends only on `objs` for domain objects and error handling infrastructure, plus `errmeta_derive` for error metadata generation
- **Process Management**: Integrates tokio async runtime with system process spawning and monitoring capabilities
- **HTTP Client Layer**: Uses reqwest for both API proxying to llama.cpp servers and GitHub release binary downloading
- **Build System Integration**: Sophisticated build-time binary management with cross-platform executable downloading and local build orchestration

## Cross-Crate Integration Patterns

### Integration with objs Crate

The crate deeply integrates with the objs crate's error handling infrastructure through `ServerError` enum implementation of the `AppError` trait, enabling localized error messages and consistent error propagation across BodhiApp. The integration includes automatic error conversion from standard library and reqwest errors using the `impl_error_from!` macro pattern, ensuring seamless error handling coordination.

### Service Layer Integration Points

Higher-level services integrate with this crate through the `Server` trait abstraction, enabling dependency injection patterns and comprehensive testing through mockall-generated mocks. The trait design supports both boxed trait objects for dynamic dispatch and direct implementation usage, providing flexibility for different service coordination patterns.

### Build System Coordination

The build system coordinates with BodhiApp's overall build infrastructure through environment variable configuration and Makefile integration, supporting both CI/CD automated binary downloading and local development build workflows. The system uses file locking to coordinate concurrent builds and integrates with the project's cross-platform build strategy.

## Important Constraints

### Process Lifecycle Management Constraints

The crate enforces strict process lifecycle management with automatic cleanup through Drop trait implementation, ensuring no orphaned processes remain after server instances are dropped. Health checking uses a fixed 300-second timeout with 1-second polling intervals, requiring llama.cpp servers to respond to `/health` endpoint within this timeframe for successful startup validation.

### Platform and Acceleration Constraints

Platform support is limited to specific target architectures with corresponding acceleration variants: macOS aarch64 supports Metal and CPU, Linux x86_64 supports CPU and CUDA, and Windows x86_64 supports CPU only. The build system requires specific binary naming conventions and GitHub release asset availability for automated downloading in CI/CD environments.

### Network and Security Constraints

All HTTP communication is constrained to localhost binding for security, with automatic port selection to avoid conflicts and TCP optimizations for connection pooling. The HTTP client uses fixed timeout and keepalive settings optimized for local llama.cpp server communication patterns.

### Build System Constraints

The build system uses file locking (`bodhi-build.lock`) to coordinate concurrent builds and requires specific environment variables for CI/CD configuration. Binary downloading depends on GitHub API availability and requires proper authentication tokens for release access, with fallback to local Makefile-based builds for development environments.

## Test Utils Architecture

### Model Fixture Management

The test_utils module provides sophisticated model fixture management through rstest fixtures that handle Hugging Face model cache integration. The `llama2_7b` fixture provides automatic model path resolution and validation, ensuring test models exist before test execution, while the `llama2_7b_str` fixture provides string path conversion for tests requiring string-based model paths.

### HTTP Response Mocking Patterns

The `mock_response` function enables comprehensive HTTP response mocking for unit tests by creating reqwest Response objects from hyper response builders. This pattern allows tests to simulate llama.cpp server responses without requiring actual server processes, enabling fast and reliable unit testing of HTTP client functionality.

### Integration Test Patterns

Integration tests demonstrate sophisticated server lifecycle testing using real llama.cpp executables and models, with rstest fixtures managing server startup and cleanup. The tests validate both streaming and non-streaming chat completions, demonstrating proper request/response handling and server process management in realistic scenarios using actual Phi-4 Mini Instruct models from Hugging Face cache.
