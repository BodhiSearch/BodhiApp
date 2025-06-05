# objs - Domain Objects and Types

## Overview

The `objs` crate serves as the foundation of the BodhiApp architecture, providing shared domain objects, data structures, and type definitions used across all other crates. It implements the core domain model and ensures type safety throughout the application.

## Purpose

- **Domain Modeling**: Defines core business entities and their relationships
- **Type Safety**: Provides strongly-typed interfaces for all data structures
- **Error Handling**: Centralized error types with metadata and localization
- **API Contracts**: Defines request/response types for OpenAI-compatible APIs
- **File Format Support**: GGUF file format parsing and metadata extraction
- **Localization**: Multi-language support infrastructure

## Key Components

### Core Domain Objects

#### Alias Management (`alias.rs`)
- Model alias definitions and management
- Mapping between user-friendly names and model identifiers
- Validation and normalization of alias names

#### Chat Templates (`chat_template.rs`, `chat_template_type.rs`)
- Chat message formatting templates
- Support for different LLM chat formats
- Template type definitions and validation

#### Repository Management (`repo.rs`)
- HuggingFace repository metadata
- Repository information and file listings
- Integration with HuggingFace Hub API

#### Hub Files (`hub_file.rs`, `remote_file.rs`)
- Remote file metadata and management
- Download progress tracking
- File integrity verification

### API Types

#### OpenAI Compatibility (`oai.rs`)
- OpenAI API request/response types
- Chat completion structures
- Model information types
- Streaming response handling

#### GPT Parameters (`gpt_params.rs`)
- LLM inference parameters
- Temperature, top-p, and other sampling settings
- Context window and token limits

### File Format Support

#### GGUF Support (`gguf/`)
- **`metadata.rs`**: GGUF file metadata parsing
- **`value.rs`**: GGUF value type handling
- **`constants.rs`**: GGUF format constants
- **`error.rs`**: GGUF-specific error types

GGUF (GPT-Generated Unified Format) is the standard format for storing LLM models, and this module provides comprehensive support for reading and interpreting GGUF files.

### Error Handling

#### Centralized Errors (`error.rs`, `error_api.rs`, `error_oai.rs`)
- **`ApiError`**: Main application error type with metadata
- **`OpenAIApiError`**: OpenAI API-specific errors
- **Error Metadata**: Rich error information with localization keys
- **Error Conversion**: Automatic conversion between error types

### Configuration and Environment

#### Environment Management (`envs.rs`)
- Environment variable handling
- Configuration validation
- Development vs production settings

#### User Roles (`role.rs`, `token_scope.rs`)
- User role definitions (Admin, PowerUser, BasicUser)
- Token scope management
- Permission-based access control

### Utilities

#### Localization (`localization_service.rs`)
- Multi-language support infrastructure
- Resource loading and management
- Fluent localization integration

#### Utilities (`utils.rs`)
- Common utility functions
- String manipulation helpers
- Validation utilities

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── alias.rs                  # Model alias management
├── chat_template.rs          # Chat formatting templates
├── chat_template_type.rs     # Template type definitions
├── envs.rs                   # Environment configuration
├── error.rs                  # Core error types
├── error_api.rs              # API error types
├── error_oai.rs              # OpenAI error types
├── gguf/                     # GGUF file format support
│   ├── mod.rs
│   ├── constants.rs
│   ├── error.rs
│   ├── metadata.rs
│   ├── value.rs
│   └── resources/            # GGUF localization resources
├── gpt_params.rs             # LLM parameters
├── hub_file.rs               # HuggingFace file metadata
├── localization_service.rs   # Localization infrastructure
├── oai.rs                    # OpenAI API types
├── remote_file.rs            # Remote file handling
├── repo.rs                   # Repository management
├── resources/                # Localization resources
│   └── en-US/
├── role.rs                   # User roles
├── test_utils/               # Testing utilities
│   ├── mod.rs
│   ├── bodhi.rs
│   ├── envs.rs
│   ├── error.rs
│   ├── hf.rs
│   ├── http.rs
│   ├── io.rs
│   ├── l10n.rs
│   ├── logs.rs
│   ├── objs.rs
│   └── test_data.rs
├── token_scope.rs            # Token permissions
└── utils.rs                  # Utility functions
```

## Key Features

### Type Safety
- Strongly-typed domain models prevent runtime errors
- Comprehensive validation at type boundaries
- Builder patterns for complex object construction

### Error Handling
- Rich error metadata with localization support
- Automatic error conversion between layers
- Structured error reporting for debugging

### Localization
- Multi-language support using Fluent
- Embedded resource files
- Runtime language switching

### Testing Support
- Comprehensive test utilities in `test_utils/`
- Mock data generation
- Test fixtures and helpers

## Dependencies

### Core Dependencies
- **serde**: Serialization/deserialization
- **utoipa**: OpenAPI documentation generation
- **derive_builder**: Builder pattern macros
- **validator**: Input validation
- **fluent**: Localization framework

### Optional Features
- **test-utils**: Testing utilities and mock data
- **dircpy**: Directory copying utilities
- **dirs**: System directory access

## Usage Patterns

The `objs` crate is used by all other crates in the workspace:

1. **Services Layer**: Uses domain objects for business logic
2. **Routes Layer**: Uses API types for request/response handling
3. **Commands Layer**: Uses configuration and error types
4. **Frontend**: Uses API types for TypeScript generation

## Integration Points

- **Error Propagation**: All errors flow through `ApiError`
- **API Documentation**: Types are documented via utoipa macros
- **Localization**: All user-facing strings use localization keys
- **Validation**: Input validation at API boundaries
