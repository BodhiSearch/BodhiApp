# CLAUDE.md - llama_server_proc

This file provides guidance to Claude Code when working with the `llama_server_proc` crate, which manages llama.cpp server processes for local LLM inference in BodhiApp.

## Purpose

The `llama_server_proc` crate provides process management and HTTP client functionality for interacting with llama.cpp server processes. It handles:

- Starting and stopping llama.cpp server processes with configurable parameters
- Proxying HTTP requests to the local LLM server 
- Building and downloading platform-specific llama-server executables
- Process lifecycle management with health checks and monitoring
- Error handling with localized error messages

## Key Components

### Server Interface (`src/server.rs`)
- `Server` trait - Async interface for LLM server operations
- `LlamaServer` - Concrete implementation managing llama.cpp server processes
- `LlamaServerArgs` - Configuration builder for server parameters
- Process lifecycle management with health checks and output monitoring

### Server Configuration
- `LlamaServerArgs` - Builder pattern for server configuration including:
  - Model path, alias, and API key
  - Network settings (host, port)
  - LLM parameters (context size, prediction length, parallel processing)
  - Feature flags (embeddings, verbose logging, web UI)

### Error Handling (`src/error.rs`)
- `ServerError` enum with localized error messages
- Integration with the `objs` crate error system
- Comprehensive error types for process management, networking, and health checks

### Build System (`src/build_envs.rs` & `build.rs`)
- Cross-platform build configuration for llama-server executables
- Automated downloading of pre-built binaries from GitHub releases
- Support for multiple variants (CPU, CUDA, Metal acceleration)
- Platform-specific executable paths and extensions

## Dependencies

### Core Dependencies
- `objs` - Domain objects and error handling infrastructure
- `errmeta_derive` - Procedural macros for error metadata
- `tokio` - Async runtime for process and network operations
- `reqwest` - HTTP client for API communication and binary downloads

### Process Management
- `derive_builder` - Builder pattern generation for configuration
- `portpicker` - Automatic port selection for server instances
- `tracing` - Structured logging with process output monitoring

### Build System Dependencies
- `anyhow` - Error handling in build scripts
- `serde_json` - JSON parsing for GitHub API responses
- `fs2` - File locking for concurrent builds
- `tempfile` - Temporary file management during downloads

## Architecture Position

The `llama_server_proc` crate sits at the infrastructure layer:
- **Foundation**: Minimal dependencies, used by higher-level services
- **Process Management**: Abstracts llama.cpp server lifecycle
- **Platform Abstraction**: Handles cross-platform executable management
- **Error Reporting**: Integrates with centralized error handling

## Usage Patterns

### Starting a Server
```rust
use llama_server_proc::{LlamaServer, LlamaServerArgsBuilder};

let args = LlamaServerArgsBuilder::default()
    .model("/path/to/model.gguf")
    .alias("my-model")
    .n_ctx(2048)
    .build()?;

let server = LlamaServer::new(&executable_path, args)?;
server.start().await?;
```

### Making API Requests
```rust
use serde_json::json;

let request = json!({
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
});

let response = server.chat_completions(&request).await?;
```

### Server Shutdown
```rust
// Automatic cleanup on drop, or explicit shutdown
let boxed_server: Box<dyn Server> = Box::new(server);
boxed_server.stop().await?;
```

## Integration Points

### With Services Layer
- Provides `Server` trait for dependency injection in business logic
- Integrates with model management and inference services
- Supports mocking via `mockall` for unit testing

### With Object Layer (`objs`)
- Uses `GptContextParams` for LLM configuration
- Integrates with centralized error handling and localization
- Shares builder patterns and validation logic

### With Build System
- `build.rs` automatically downloads platform-specific binaries
- Makefile integration for cross-platform compilation
- CI/CD integration with variant selection and caching

## Development Guidelines

### Adding New Server Features
1. Extend `LlamaServerArgs` with new configuration options
2. Update `to_args()` method to include new command-line flags
3. Add corresponding fields to the builder pattern
4. Update tests to verify new functionality

### Error Handling
- Use `ServerError` enum for all crate-specific errors
- Include localized error messages via `errmeta_derive`
- Provide context with error details for debugging
- Convert external errors using `impl_error_from!` macro

### Testing
- Use `test-utils` feature for mock implementations
- Test server lifecycle with health checks
- Verify argument serialization and process communication
- Mock HTTP responses for unit testing

## Platform Support

### Supported Platforms
- **macOS (aarch64)**: Metal and CPU acceleration
- **Linux (x86_64)**: CPU and CUDA 12.6 acceleration  
- **Windows (x86_64)**: CPU acceleration only

### Build Variants
- **CPU**: Standard CPU-only inference
- **Metal**: Apple Silicon GPU acceleration (macOS only)
- **CUDA**: NVIDIA GPU acceleration (Linux only)

## Process Management

### Health Checks
- Polls `/health` endpoint with 300-second timeout
- Exponential backoff with 1-second intervals
- Automatic process cleanup on drop or explicit shutdown

### Output Monitoring
- Captures stdout/stderr in separate threads
- Routes output to structured logging system
- Warns on process errors and startup issues

### Networking
- HTTP/1.1 client with connection pooling
- TCP keepalive and nodelay optimizations
- Localhost binding for security
- Automatic port selection to avoid conflicts

## File Outputs

- `bin/{target}/{variant}/llama-server[.exe]` - Platform-specific executables
- `bodhi-build.lock` - Build process synchronization
- Process output routed to tracing system