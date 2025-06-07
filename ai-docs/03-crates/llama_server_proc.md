# llama_server_proc - LLM Server Process Management

## Overview

The `llama_server_proc` crate provides integration with llama.cpp, managing the lifecycle of local LLM inference servers. It handles compilation, process management, and communication with llama.cpp server instances to provide local LLM capabilities for BodhiApp.

## Purpose

- **LLM Integration**: Integration with llama.cpp for local LLM inference
- **Process Management**: Lifecycle management of llama.cpp server processes
- **Build System**: Compilation and build management for llama.cpp
- **Performance Optimization**: Hardware-specific optimizations and configurations
- **Resource Management**: Memory and GPU resource management for LLM inference

## Key Components

### Server Management

#### Server Process (`server.rs`)
- **Process Lifecycle**: Start, stop, and monitor llama.cpp server processes
- **Configuration Management**: Server configuration and parameter management
- **Health Monitoring**: Server health checks and status monitoring
- **Resource Allocation**: Memory and GPU resource allocation
- **Error Recovery**: Automatic restart and error recovery mechanisms

Key features:
- Dynamic model loading and unloading
- Multi-model support with resource sharing
- Automatic scaling based on demand
- Process isolation and security

### Build System

#### Build Environment (`build_envs.rs`)
- **Compilation Management**: llama.cpp compilation and build process
- **Platform Detection**: Automatic platform and hardware detection
- **Optimization Flags**: Hardware-specific optimization configuration
- **Dependency Management**: Build dependency resolution and management
- **Cross-Platform Support**: Support for different operating systems and architectures

Build capabilities:
- CPU-optimized builds (AVX, AVX2, AVX-512)
- GPU acceleration (CUDA, OpenCL, Metal)
- Platform-specific optimizations
- Custom build configurations

### Error Handling

#### Process Errors (`error.rs`)
- **Process-Specific Errors**: llama.cpp process error handling
- **Build Errors**: Compilation and build error management
- **Runtime Errors**: LLM inference error handling
- **Resource Errors**: Memory and GPU resource error handling

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── server.rs                 # LLM server process management
├── build_envs.rs             # Build environment and compilation
├── error.rs                  # Process-specific error handling
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    └── mod.rs
```

## Build System Integration

### llama.cpp Integration
The crate includes llama.cpp as a git submodule and manages its compilation:

```
llama.cpp/                    # Git submodule
├── CMakeLists.txt
├── src/
├── examples/
└── models/

build.rs                      # Build script for compilation
Makefile                      # Platform-specific build rules
Makefile.win.mk              # Windows-specific build rules
scripts/                     # Build automation scripts
bin/                         # Compiled binaries
```

### Build Configuration
The build system supports various configurations:

```rust
pub struct BuildConfig {
    pub enable_cuda: bool,
    pub enable_opencl: bool,
    pub enable_metal: bool,
    pub enable_avx: bool,
    pub enable_avx2: bool,
    pub enable_avx512: bool,
    pub optimization_level: OptimizationLevel,
    pub target_arch: TargetArch,
}
```

### Platform Support
- **Linux**: Full support with GPU acceleration
- **macOS**: Metal GPU acceleration support
- **Windows**: CUDA and OpenCL support
- **ARM**: ARM64 optimization support

## Server Process Management

### Process Lifecycle
```rust
pub struct LlamaServer {
    process: Child,
    config: ServerConfig,
    status: ServerStatus,
}

impl LlamaServer {
    pub async fn start(config: ServerConfig) -> Result<Self, ProcessError>;
    pub async fn stop(&mut self) -> Result<(), ProcessError>;
    pub async fn restart(&mut self) -> Result<(), ProcessError>;
    pub fn status(&self) -> ServerStatus;
    pub async fn health_check(&self) -> Result<HealthStatus, ProcessError>;
}
```

### Configuration Management
```rust
pub struct ServerConfig {
    pub model_path: PathBuf,
    pub context_size: u32,
    pub batch_size: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub host: String,
    pub port: u16,
    pub memory_limit: Option<u64>,
}
```

### Resource Management
- **Memory Allocation**: Dynamic memory allocation based on model size
- **GPU Management**: GPU memory management and layer distribution
- **CPU Optimization**: Thread allocation and CPU affinity
- **Model Caching**: Intelligent model caching and preloading

## Key Features

### Hardware Acceleration
- **CUDA Support**: NVIDIA GPU acceleration
- **OpenCL Support**: Cross-platform GPU acceleration
- **Metal Support**: Apple Silicon GPU acceleration
- **CPU Optimization**: Advanced CPU instruction set utilization

### Model Management
- **Dynamic Loading**: Load and unload models at runtime
- **Multi-Model Support**: Run multiple models simultaneously
- **Model Switching**: Fast model switching with minimal downtime
- **Memory Optimization**: Efficient memory usage across models

### Performance Optimization
- **Batch Processing**: Efficient batch inference processing
- **Context Caching**: Intelligent context caching for performance
- **Memory Mapping**: Memory-mapped model loading
- **Quantization Support**: Support for quantized models (Q4, Q8, etc.)

### Monitoring and Diagnostics
- **Performance Metrics**: Real-time performance monitoring
- **Resource Usage**: Memory and GPU usage tracking
- **Error Diagnostics**: Comprehensive error reporting and diagnostics
- **Health Checks**: Continuous health monitoring

## Dependencies

### Core Dependencies
- **objs**: Domain objects and error types
- **tokio**: Async runtime for process management
- **serde**: Configuration serialization

### Build Dependencies
- **cc**: C/C++ compilation
- **cmake**: CMake build system integration
- **pkg-config**: System library detection

### System Dependencies
- **libc**: System library integration
- **libloading**: Dynamic library loading

## Usage Patterns

### Server Startup
```rust
let config = ServerConfig {
    model_path: PathBuf::from("/path/to/model.gguf"),
    context_size: 2048,
    batch_size: 512,
    threads: 8,
    gpu_layers: 32,
    host: "127.0.0.1".to_string(),
    port: 8080,
    memory_limit: Some(8 * 1024 * 1024 * 1024), // 8GB
};

let server = LlamaServer::start(config).await?;
```

### Health Monitoring
```rust
let health = server.health_check().await?;
match health.status {
    HealthStatus::Healthy => println!("Server is healthy"),
    HealthStatus::Degraded => println!("Server performance degraded"),
    HealthStatus::Unhealthy => println!("Server needs restart"),
}
```

### Resource Monitoring
```rust
let metrics = server.get_metrics().await?;
println!("Memory usage: {}MB", metrics.memory_usage / 1024 / 1024);
println!("GPU usage: {}%", metrics.gpu_utilization);
println!("Requests per second: {}", metrics.requests_per_second);
```

## Build Process

### Compilation Steps
1. **Platform Detection**: Detect target platform and available hardware
2. **Dependency Check**: Verify build dependencies (CMake, compilers)
3. **Configuration**: Generate build configuration based on hardware
4. **Compilation**: Compile llama.cpp with optimizations
5. **Binary Installation**: Install compiled binaries to appropriate locations

### Build Optimization
```rust
// Automatic hardware detection and optimization
let build_config = BuildConfig::detect_hardware()?;
let optimized_binary = compile_llama_cpp(build_config).await?;
```

### Cross-Platform Builds
- **Linux**: GCC/Clang with CUDA/OpenCL support
- **macOS**: Clang with Metal support
- **Windows**: MSVC with CUDA support
- **ARM**: Cross-compilation support for ARM64

## Error Handling

### Process Errors
- **Startup Failures**: Server startup and initialization errors
- **Runtime Errors**: Inference and processing errors
- **Resource Errors**: Memory and GPU allocation errors
- **Communication Errors**: IPC and network communication errors

### Build Errors
- **Compilation Errors**: C++ compilation failures
- **Dependency Errors**: Missing build dependencies
- **Configuration Errors**: Invalid build configurations
- **Platform Errors**: Unsupported platform or hardware

## Testing Support

### Process Testing
- **Mock Servers**: Mock llama.cpp server implementations
- **Process Simulation**: Simulate process lifecycle events
- **Resource Testing**: Test resource allocation and management
- **Error Simulation**: Simulate various error conditions

### Build Testing
- **Cross-Platform Testing**: Test builds on different platforms
- **Configuration Testing**: Test different build configurations
- **Performance Testing**: Benchmark different optimization levels

## Integration Points

- **Services Layer**: Provides LLM inference capabilities to services
- **Routes Layer**: Serves inference requests through HTTP APIs
- **Configuration**: Integrates with application configuration system
- **Monitoring**: Provides metrics for application monitoring

## Performance Considerations

### Memory Management
- **Model Loading**: Efficient model loading and memory mapping
- **Context Management**: Intelligent context window management
- **Garbage Collection**: Proper cleanup of inference resources
- **Memory Pooling**: Reuse memory allocations for performance

### GPU Optimization
- **Layer Distribution**: Optimal GPU layer distribution
- **Memory Transfer**: Efficient CPU-GPU memory transfers
- **Batch Processing**: GPU-optimized batch processing
- **Multi-GPU Support**: Support for multiple GPU configurations

## Future Extensions

The llama_server_proc crate is designed to support:
- **Distributed Inference**: Multi-node inference clusters
- **Model Quantization**: Runtime model quantization
- **Advanced Caching**: Sophisticated caching strategies
- **Custom Kernels**: Custom GPU kernels for specific operations
- **Model Compilation**: Ahead-of-time model compilation
