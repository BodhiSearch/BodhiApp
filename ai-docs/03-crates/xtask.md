# xtask - Build Automation and Development Tasks

## Overview

The `xtask` crate provides build automation and development task management for BodhiApp. It implements custom build scripts, code generation, and development workflow automation using the xtask pattern, which is a Rust community convention for project-specific tooling.

## Purpose

- **Build Automation**: Automate complex build processes and workflows
- **Code Generation**: Generate TypeScript types and OpenAPI documentation
- **Development Tools**: Provide development utilities and helpers
- **CI/CD Integration**: Support continuous integration and deployment workflows
- **Project Maintenance**: Automate project maintenance tasks

## Key Components

### OpenAPI Generation (`openapi.rs`)
- **OpenAPI Spec Generation**: Generate OpenAPI specifications from Rust code
- **Documentation Export**: Export API documentation in various formats
- **Validation**: Validate OpenAPI specifications for correctness
- **Version Management**: Manage API version compatibility

Key features:
- Automatic OpenAPI 3.1 specification generation
- Integration with utoipa annotations
- Multi-format export (JSON, YAML)
- API documentation validation

### TypeScript Generation (`typescript.rs`)
- **Type Generation**: Generate TypeScript types from Rust structs
- **Client Generation**: Generate TypeScript API clients
- **Type Safety**: Ensure type safety between frontend and backend
- **Synchronization**: Keep frontend types synchronized with backend changes

Key features:
- Automatic TypeScript type generation
- API client generation with proper typing
- Frontend-backend type synchronization
- Custom type mapping and transformations

### Main Task Runner (`main.rs`)
- **Task Orchestration**: Coordinate multiple build tasks
- **Command-Line Interface**: Provide CLI for development tasks
- **Error Handling**: Comprehensive error handling for build processes
- **Logging**: Detailed logging for build process debugging

## Directory Structure

```
src/
├── main.rs                   # Main task runner and CLI
├── openapi.rs                # OpenAPI specification generation
└── typescript.rs             # TypeScript type and client generation
```

## Available Tasks

### Code Generation Tasks

#### OpenAPI Generation
```bash
# Generate OpenAPI specification
cargo xtask openapi

# Generate with specific output format
cargo xtask openapi --format json
cargo xtask openapi --format yaml

# Validate existing specification
cargo xtask openapi --validate
```

#### TypeScript Generation
```bash
# Generate TypeScript types
cargo xtask typescript

# Generate API client
cargo xtask typescript --client

# Generate with custom output directory
cargo xtask typescript --output ./frontend/src/types
```

### Development Tasks

#### Full Build
```bash
# Complete build with all code generation
cargo xtask build

# Build with specific features
cargo xtask build --features production
```

#### Testing Tasks
```bash
# Run all tests with code generation
cargo xtask test

# Run integration tests
cargo xtask test --integration

# Run with coverage
cargo xtask test --coverage
```

#### Cleanup Tasks
```bash
# Clean generated files
cargo xtask clean

# Clean and regenerate everything
cargo xtask clean --all
cargo xtask build
```

## Implementation Details

### OpenAPI Generation

#### Specification Generation
```rust
pub async fn generate_openapi_spec() -> Result<(), XTaskError> {
    // Start the server to extract OpenAPI spec
    let server = start_test_server().await?;
    
    // Fetch OpenAPI specification
    let spec = fetch_openapi_spec(&server).await?;
    
    // Validate specification
    validate_openapi_spec(&spec)?;
    
    // Write to output files
    write_openapi_json(&spec, "openapi.json").await?;
    write_openapi_yaml(&spec, "openapi.yaml").await?;
    
    Ok(())
}
```

#### Documentation Generation
```rust
pub async fn generate_api_docs() -> Result<(), XTaskError> {
    let spec = load_openapi_spec().await?;
    
    // Generate HTML documentation
    generate_html_docs(&spec, "docs/api").await?;
    
    // Generate Markdown documentation
    generate_markdown_docs(&spec, "docs/api.md").await?;
    
    Ok(())
}
```

### TypeScript Generation

#### Type Generation
```rust
pub async fn generate_typescript_types() -> Result<(), XTaskError> {
    let spec = load_openapi_spec().await?;
    
    // Generate TypeScript interfaces
    let types = generate_ts_types(&spec)?;
    
    // Write to output file
    write_typescript_file(&types, "frontend/src/types/api.ts").await?;
    
    // Generate index file
    generate_types_index("frontend/src/types").await?;
    
    Ok(())
}
```

#### Client Generation
```rust
pub async fn generate_api_client() -> Result<(), XTaskError> {
    let spec = load_openapi_spec().await?;
    
    // Generate API client
    let client = generate_ts_client(&spec)?;
    
    // Write client file
    write_typescript_file(&client, "frontend/src/api/client.ts").await?;
    
    // Generate client utilities
    generate_client_utils("frontend/src/api").await?;
    
    Ok(())
}
```

## Task Configuration

### Configuration File
```toml
# xtask.toml
[openapi]
output_dir = "docs/api"
formats = ["json", "yaml"]
validate = true

[typescript]
output_dir = "frontend/src/types"
generate_client = true
client_output = "frontend/src/api"

[build]
features = ["production"]
target_dir = "target"

[test]
coverage = true
integration = true
```

### Environment Variables
```bash
# OpenAPI configuration
XTASK_OPENAPI_OUTPUT_DIR=./docs/api
XTASK_OPENAPI_FORMAT=json

# TypeScript configuration
XTASK_TS_OUTPUT_DIR=./frontend/src/types
XTASK_TS_GENERATE_CLIENT=true

# Build configuration
XTASK_BUILD_FEATURES=production
XTASK_TARGET_DIR=./target
```

## Dependencies

### Core Dependencies
- **xtaskops**: XTask operations and utilities
- **tokio**: Async runtime for task execution
- **clap**: Command-line argument parsing
- **serde**: Configuration serialization

### Code Generation
- **utoipa**: OpenAPI specification extraction
- **serde_json**: JSON processing
- **serde_yaml**: YAML processing
- **reqwest**: HTTP client for API calls

### File Operations
- **fs_extra**: Enhanced file system operations
- **walkdir**: Directory traversal
- **tempfile**: Temporary file management

## Usage Examples

### Basic Usage
```bash
# Generate all code
cargo xtask generate

# Build everything
cargo xtask build

# Run tests with generated code
cargo xtask test
```

### Advanced Usage
```bash
# Generate OpenAPI with validation
cargo xtask openapi --validate --format json

# Generate TypeScript with custom output
cargo xtask typescript --output ./custom/types --client

# Full development workflow
cargo xtask clean
cargo xtask generate
cargo xtask build --features development
cargo xtask test --coverage
```

### CI/CD Integration
```yaml
# GitHub Actions workflow
name: Build and Test
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Generate code
        run: cargo xtask generate
      - name: Build
        run: cargo xtask build
      - name: Test
        run: cargo xtask test --coverage
```

## Error Handling

### Task Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum XTaskError {
    #[error("OpenAPI generation failed: {0}")]
    OpenApiGeneration(String),
    
    #[error("TypeScript generation failed: {0}")]
    TypeScriptGeneration(String),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
```

### Error Recovery
```rust
pub async fn generate_with_retry() -> Result<(), XTaskError> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match generate_openapi_spec().await {
            Ok(_) => return Ok(()),
            Err(e) if attempts < max_attempts - 1 => {
                eprintln!("Generation failed, retrying: {}", e);
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

## Integration Points

- **Build System**: Integrates with Cargo build process
- **Frontend**: Generates types for React frontend
- **CI/CD**: Provides automation for continuous integration
- **Documentation**: Generates API documentation
- **Development**: Supports development workflow automation

## Performance Optimization

### Incremental Generation
- **Change Detection**: Only regenerate when source changes
- **Caching**: Cache generated artifacts for faster builds
- **Parallel Processing**: Parallel task execution where possible
- **Dependency Tracking**: Track dependencies for minimal rebuilds

### Build Optimization
```rust
pub async fn incremental_generate() -> Result<(), XTaskError> {
    let source_hash = calculate_source_hash().await?;
    let cached_hash = load_cached_hash().await.ok();
    
    if Some(source_hash) == cached_hash {
        println!("No changes detected, skipping generation");
        return Ok(());
    }
    
    generate_all().await?;
    save_cached_hash(source_hash).await?;
    
    Ok(())
}
```

## Future Extensions

The xtask crate is designed to support:
- **Custom Generators**: Plugin system for custom code generators
- **Advanced Validation**: Enhanced validation and linting
- **Performance Monitoring**: Build performance monitoring and optimization
- **Cross-Platform Support**: Enhanced cross-platform build support
- **Integration Testing**: Automated integration test generation
