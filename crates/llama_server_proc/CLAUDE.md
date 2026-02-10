# CLAUDE.md

This file provides guidance to Claude Code when working with the llama_server_proc crate.

See [crates/llama_server_proc/PACKAGE.md](crates/llama_server_proc/PACKAGE.md) for implementation details.

## Purpose

The `llama_server_proc` crate provides comprehensive process management and HTTP client functionality for interacting with llama.cpp server processes in BodhiApp's local LLM inference infrastructure. It serves as the foundational layer for managing external llama.cpp server processes with sophisticated lifecycle management, health monitoring, and cross-platform binary distribution.

## Key Domain Architecture

### Process Management System

The crate implements a sophisticated process lifecycle management system that handles llama.cpp server processes with comprehensive monitoring and health checking. The architecture provides async trait-based abstraction (`Server` trait) enabling dependency injection and testing through mockall integration, while the concrete `LlamaServer` implementation manages actual process spawning, monitoring, and cleanup with automatic resource management through Drop trait implementation.

The `Server` trait exposes two stop methods: `stop(self: Box<Self>)` for trait-object consumers and `stop_unboxed(self)` for direct struct usage. This dual-method design addresses Rust's limitation where `self` methods cannot be called on `Box<dyn Trait>` -- higher-level services hold `Box<dyn Server>` and call `stop()`, while tests and direct consumers can call `stop_unboxed()` without boxing overhead.

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

## Cross-Crate Coordination Patterns

### Integration with objs Crate

The crate deeply integrates with the objs crate's error handling infrastructure through `ServerError` enum implementation of the `AppError` trait, enabling user-friendly error messages via thiserror templates and consistent error propagation across BodhiApp. The integration includes automatic error conversion from standard library and reqwest errors using the `impl_error_from!` macro pattern, ensuring seamless error handling coordination.

### Service Layer Integration Points

Higher-level services integrate with this crate through the `Server` trait abstraction, enabling dependency injection patterns and comprehensive testing through mockall-generated mocks. The trait design supports both boxed trait objects for dynamic dispatch and direct implementation usage, providing flexibility for different service coordination patterns.

## Build System Orchestration Patterns

The build system coordinates with BodhiApp's overall build infrastructure through environment variable configuration and Makefile integration, supporting both CI/CD automated binary downloading and local development build workflows. The system uses file locking to coordinate concurrent builds and integrates with the project's cross-platform build strategy.

## Testing Architecture

### Two-Tier Test Strategy

The crate employs two distinct test tiers with fundamentally different purposes:

1. **Integration tests with large models** (`tests/test_server_proc.rs`): Tests use Qwen3-1.7B from the system HuggingFace cache (`~/.cache/huggingface/hub`). These tests validate full chat completion flows (streaming and non-streaming) against a real llama.cpp server, requiring significant model download (~1.7GB) and startup time. They require the `HF_HOME` or default HuggingFace cache to contain the model.

2. **Live tests with bundled lightweight models** (`tests/test_live_server_proc.rs`): Tests use Llama-68M (afrideva/Llama-68M-Chat-v1-GGUF), a tiny 68M parameter model stored directly in the test data directory at `tests/data/live/huggingface/`. These tests validate core process lifecycle (start/stop) without needing external model downloads. The model's small size enables fast startup and minimal resource usage, making these tests suitable for CI/CD environments.

### Why Two Tiers

The integration tests (`test_server_proc.rs`) validate end-to-end inference correctness with production-sized models, covering streaming SSE parsing, response format validation, and model alias propagation. The live tests (`test_live_server_proc.rs`) focus on process management correctness -- can the binary be found, spawned, health-checked, and stopped cleanly? By using a bundled 68M model, live tests avoid external dependencies while still exercising the real llama.cpp binary.

### Live Test Data Layout

The `tests/data/live/huggingface/` directory mirrors the standard HuggingFace cache structure:
- `hub/models--afrideva--Llama-68M-Chat-v1-GGUF/` -- GGUF quantized model for process lifecycle tests
- `hub/models--Felladrin--Llama-68M-Chat-v1/` -- Original safetensors model (source reference)
- `hub/models--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF/` -- Additional test model
- Snapshots use symlinks to blobs, exactly as HuggingFace CLI downloads them

### Test Binary Resolution

Both test tiers resolve the llama-server binary using compile-time constants from `build_envs.rs`:
- `BUILD_TARGET` -- platform triple (e.g., `aarch64-apple-darwin`)
- `DEFAULT_VARIANT` -- acceleration variant (e.g., `metal`, `cpu`)
- `EXEC_NAME` -- executable name (e.g., `llama-server`)

The binary is expected at `bin/{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}` relative to the crate manifest directory.

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
