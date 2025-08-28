# CLAUDE.md

This file provides guidance to Claude Code when working with the errmeta_derive crate.

## Purpose

The `errmeta_derive` crate is a procedural macro library that automatically implements error metadata functionality for Rust enums and structs in BodhiApp. It provides the foundation for structured error handling across all service layers by generating consistent metadata extraction methods.

Key capabilities:
- The `#[derive(ErrorMeta)]` macro for automatic metadata generation
- Support for custom error codes, types, and localization arguments  
- Integration with transparent error wrapping via `#[error(transparent)]`
- Compile-time validation of error metadata attributes
- Flexible trait implementation via `trait_to_impl` parameter

## Key Domain Architecture

### Error Metadata Generation System
The macro generates three core methods that provide structured error information:
- `error_type()` - Returns categorized error type for service-level error handling
- `code()` - Provides localization keys for user-facing error messages  
- `args()` - Extracts structured data from error fields for message templating

### Attribute Processing Architecture
- `#[error_meta(...)]` attributes for customizing error metadata behavior
- Expression evaluation system supporting string literals, function calls, and enum variants
- Transparent error delegation system with configurable argument handling
- Compile-time validation ensuring required attributes are present

### Code Generation Patterns
- Pattern matching generation for all Rust field types (named, unnamed, unit)
- Automatic snake_case conversion for default error codes
- Integration with `#[error(transparent)]` for error wrapping scenarios
- Trait implementation flexibility via `trait_to_impl` parameter

## Architecture Position

The `errmeta_derive` crate operates at the foundational macro layer of BodhiApp's error handling architecture:

- **Foundation Layer**: Provides compile-time code generation for error metadata extraction
- **Zero Runtime Cost**: All processing occurs at compile time with no runtime overhead
- **Cross-Crate Integration**: Used by `objs` crate and service layers for consistent error handling
- **Localization Foundation**: Generates structured data required for error message localization

## Cross-Crate Integration Patterns

### Integration with Object Layer (`objs`)
- Error enums in the `objs` crate derive `ErrorMeta` for centralized error handling
- Provides structured error data for HTTP status code mapping and API responses
- Integrates with localization service for user-facing error message formatting

### Service Layer Coordination
- Service-specific error types derive `ErrorMeta` for consistent error reporting across business logic
- Business logic errors include relevant context via automatic args extraction
- Integration with tracing and logging systems via structured metadata

### Transparent Error Wrapping System
- Supports `#[error(transparent)]` for delegating error metadata to wrapped errors
- Configurable argument delegation via `args_delegate` attribute
- Enables clean error propagation across service boundaries while maintaining metadata

## Important Constraints

### Compile-Time Validation Requirements
- Enum variants without `#[error(transparent)]` must specify `error_type` attribute
- Struct types must include `error_type` in their `#[error_meta(...)]` attribute
- Union types are not supported and will cause compilation failure
- Invalid expressions in attributes trigger compile-time errors

### Attribute Expression Support
- Supports string literals: `error_type = "ValidationError"`
- Supports function calls: `error_type = get_error_type()`
- Supports enum variants: `error_type = ErrorType::Validation`
- Supports complex expressions: `code = self.generate_code()`

### Transparent Error Delegation Rules
- `#[error(transparent)]` variants automatically delegate to wrapped error's metadata methods
- `args_delegate = false` overrides delegation for args(), using error string instead
- Transparent variants can override `error_type` and `code` while maintaining args delegation
- Mixed transparent and non-transparent variants are supported in the same enum


## Macro Testing Architecture

### Compile-Time Validation Testing
The crate uses `trybuild` for compile-time error validation, ensuring that invalid macro usage produces appropriate compiler errors:
- Missing `error_type` attributes on enum variants trigger compilation failures
- Union types are rejected with clear error messages  
- Invalid attribute expressions are caught during macro expansion

### Runtime Behavior Testing
Comprehensive test suite using `rstest` for parameterized testing of generated code:
- Tests all field patterns (named, unnamed, unit) for both enums and structs
- Validates transparent error delegation with and without `args_delegate`
- Integration testing with `thiserror` and `strum` for real-world usage patterns
- Verification of automatic snake_case code generation

### Test Data Structures
The test suite includes a mock `ErrorMetas` struct that mirrors the expected interface for integration with the `objs` crate, enabling validation of the complete error metadata extraction workflow.

## Procedural Macro Implementation Patterns

### Token Stream Processing
- Uses `syn` with full feature support for comprehensive Rust syntax parsing
- `quote!` macro for generating clean, properly escaped Rust code
- `proc-macro2::TokenStream` for internal token manipulation

### Pattern Matching Code Generation
- Generates match arms for all enum variants with appropriate field destructuring
- Handles named fields with `{ field1, field2 }` patterns
- Handles unnamed fields with `(var_0, var_1)` patterns  
- Handles unit variants with empty patterns

### Expression Evaluation System
- Supports arbitrary Rust expressions in `error_type` and `code` attributes
- Uses `syn::Expr` parsing for flexible attribute value handling
- Generates code that evaluates expressions at runtime while maintaining compile-time validation

## Integration with BodhiApp Error Handling

### Foundation for Structured Errors
The macro provides the foundation for BodhiApp's structured error handling by ensuring all error types can provide:
- Categorized error types for service-level error routing
- Localization keys for user-facing error messages
- Structured arguments for error message templating

### Service Boundary Error Propagation
Transparent error support enables clean error propagation across service boundaries while maintaining error metadata, crucial for BodhiApp's multi-service architecture.