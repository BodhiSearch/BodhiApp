# CLAUDE.md - objs

This file provides guidance to Claude Code when working with the `objs` crate, which contains domain objects, error handling infrastructure, and shared types for BodhiApp.

## Purpose

The `objs` crate is the foundational layer providing:

- **Domain Objects**: Core business entities like models, repositories, and contexts
- **Error Handling**: Centralized error system with localization support
- **OpenAI Compatibility**: Request/response types for OpenAI API compatibility
- **Configuration**: Parameter validation and CLI argument parsing
- **Utilities**: Common functionality used across the application
- **Localization**: Multi-language error message support

## Key Components

### Error Handling System (`src/error/`)
- `ErrorType` enum for HTTP status code mapping and error categorization
- `AppError` trait providing consistent error metadata interface
- `ErrorMessage` struct for JSON error responses
- Specialized error types for validation, authentication, and service errors
- Integration with `errmeta_derive` for automatic error metadata generation

### Domain Objects
- `GptContextParams` - LLM context configuration (threads, context size, prediction tokens)
- `OAIRequestParams` - OpenAI API request parameters with validation
- `Alias`, `Repo`, `HubFile` - Model management and repository types
- Scope types (`UserScope`, `TokenScope`, `ResourceScope`) for authorization

### Localization Service (`src/localization_service.rs`)
- `FluentLocalizationService` - Thread-safe localization with Fluent templates
- Support for multiple locales (en-US, fr-FR) with fallback mechanisms
- Dynamic resource loading from embedded directories
- Singleton pattern for global access

### OpenAI Compatibility (`src/oai.rs`)
- Parameter validation with range checking (-2.0 to 2.0, 0.0 to 1.0, etc.)
- Integration with `async-openai` types for request transformation
- CLI argument parsing for OpenAI-compatible parameters

### Utilities (`src/utils.rs`)
- Safe filename generation with illegal character replacement
- Default value checking for serialization optimization
- Regular expressions for input sanitization

## Dependencies

### Core Infrastructure
- `errmeta_derive` - Procedural macros for error metadata
- `thiserror` - Error trait derivation  
- `serde` - Serialization/deserialization
- `validator` - Data validation with custom error messages

### CLI and Configuration
- `clap` - Command-line argument parsing with derive macros
- `derive_builder` - Builder pattern generation
- `strum` - Enum string conversions and display

### Localization  
- `fluent` - Localization message formatting
- `unic-langid` - Language identifier parsing
- `include_dir` - Embedded resource directories

### OpenAI Integration
- `async-openai` - OpenAI API types and client integration
- `utoipa` - OpenAPI schema generation

### HTTP and API
- `axum` - HTTP server framework integration
- `reqwest` - HTTP client capabilities

## Architecture Position

The `objs` crate sits at the foundation of BodhiApp:
- **Foundation Layer**: Provides core types and error handling used by all other crates
- **Domain Model**: Defines business entities and their relationships
- **Integration Point**: Bridges external APIs (OpenAI) with internal representations
- **Configuration Hub**: Centralizes parameter validation and CLI parsing

## Usage Patterns

### Error Handling
```rust
use objs::{AppError, ErrorType, ObjValidationError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MyServiceError {
    #[error("invalid_model")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    InvalidModel(String),
    
    #[error(transparent)]
    ValidationError(#[from] ObjValidationError),
}
```

### Configuration Building
```rust
use objs::{GptContextParams, OAIRequestParams};

let gpt_params = GptContextParams {
    n_ctx: Some(2048),
    n_predict: Some(100),
    n_parallel: Some(4),
    ..Default::default()
};

let mut oai_request = CreateChatCompletionRequest::default();
let oai_params = OAIRequestParams {
    temperature: Some(0.7),
    max_tokens: Some(150),
    ..Default::default()
};
oai_params.update(&mut oai_request);
```

### Localization
```rust
use objs::{FluentLocalizationService, LocalizationService};
use std::collections::HashMap;

let service = FluentLocalizationService::get_instance();
let args = HashMap::from([("name".to_string(), "Alice".to_string())]);
let message = service.get_message(&locale, "welcome_message", Some(args))?;
```

## Integration Points

### With Higher-Level Crates
- **Services Layer**: Uses domain objects and error types for business logic
- **Server Layer**: Integrates error handling with HTTP responses  
- **CLI Layer**: Uses parameter structs for command-line interfaces
- **API Routes**: Leverages OpenAI compatibility types for endpoint implementations

### With External Systems
- **OpenAI API**: Direct compatibility with `async-openai` request/response types
- **llama.cpp**: Parameter translation for local LLM inference
- **HTTP Clients**: Error handling for external API communications

## Error System Design

### Error Categories
- **Validation Errors**: Input validation failures with structured field information
- **Authentication Errors**: User identity and authorization failures
- **Service Errors**: External service communication failures
- **Internal Errors**: Application logic and system errors

### Localization Support
- Error codes map to Fluent message templates
- Arguments extracted from error data for message formatting
- Multi-language support with fallback to English
- Thread-safe concurrent access to localization resources

## Configuration Validation

### Parameter Ranges
- `temperature`: 0.0 to 2.0 for output randomness control
- `frequency_penalty`: -2.0 to 2.0 for repetition control
- `presence_penalty`: -2.0 to 2.0 for topic diversity
- `top_p`: 0.0 to 1.0 for nucleus sampling

### CLI Integration
- Automatic help text generation from parameter descriptions
- Type-safe argument parsing with custom validators
- Optional parameter handling with sensible defaults

## Development Guidelines

### Adding New Domain Objects
1. Implement `Serialize`, `Deserialize`, and `Debug` traits
2. Add `Builder` derive for complex configuration types  
3. Include `ToSchema` for OpenAPI documentation
4. Add validation constraints where appropriate

### Error Handling Best Practices
- Use `ErrorType` enum for consistent HTTP status mapping
- Implement `AppError` trait for all error types via `errmeta_derive`
- Provide localized error messages with argument extraction
- Group related errors into focused enums

### Testing Patterns
- Use `rstest` for parameterized test cases
- Test validation ranges thoroughly with boundary conditions
- Verify localization with multiple languages and argument combinations
- Test error serialization and deserialization

## File Structure

```
src/
├── error/           # Error handling infrastructure
│   ├── common.rs    # Common error types (EntityError, ValidationError)
│   ├── l10n.rs      # Localization-specific errors
│   └── objs.rs      # Object validation and service errors
├── gguf/           # GGUF model file handling
├── log.rs          # Logging configuration
├── oai.rs          # OpenAI compatibility types
├── gpt_params.rs   # LLM context parameters
└── utils.rs        # Utility functions
```

## Localization Resources

- Embedded via `include_dir!` macro for zero-dependency deployment
- Fluent template files (`.ftl`) organized by language identifier
- Automatic resource discovery and loading
- Thread-safe concurrent access with `RwLock` protection