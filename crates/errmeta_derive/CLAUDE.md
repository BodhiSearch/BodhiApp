# CLAUDE.md - errmeta_derive

This file provides guidance to Claude Code when working with the `errmeta_derive` crate, which provides procedural macros for adding metadata to error types in BodhiApp.

## Purpose

The `errmeta_derive` crate is a procedural macro library that automatically implements error metadata functionality for Rust enums and structs. It provides:

- The `#[derive(ErrorMeta)]` macro for automatic metadata generation
- Support for custom error codes, types, and localization arguments
- Integration with transparent error wrapping via `#[error(transparent)]`
- Compile-time validation of error metadata attributes
- Flexible trait implementation via `trait_to_impl` parameter

## Key Components

### Main Derive Macro (`#[derive(ErrorMeta)]`)
- Generates `error_type()`, `code()`, and `args()` methods for error types
- Supports both enums and structs with different field patterns
- Integrates with existing error handling patterns like `thiserror`
- Provides automatic snake_case code generation from type/variant names

### Attribute Parsing
- `#[error_meta(...)]` attributes for customizing error metadata
- Support for expressions, string literals, and enum values
- Optional trait implementation via `trait_to_impl`
- Transparent error delegation with `args_delegate` control

### Code Generation
- Pattern matching for enum variants with named, unnamed, and unit fields
- Automatic field extraction for error arguments
- Integration with transparent errors via delegation
- Compile-time error reporting for missing required attributes

## Dependencies

### Procedural Macro Infrastructure
- `proc-macro2` - TokenStream manipulation and code generation
- `quote` - Rust code generation with proper escaping
- `syn` - Rust syntax tree parsing with full feature support
- `convert_case` - String case conversion for snake_case codes

### Development Dependencies
- `rstest` - Parameterized testing for macro validation
- `trybuild` - Compile-time macro error testing
- `pretty_assertions` - Enhanced assertion formatting
- `thiserror` and `strum` - Integration testing with common error libraries

## Architecture Position

The `errmeta_derive` crate sits at the foundation layer:
- **Foundation**: Provides core macro infrastructure for error handling
- **Code Generation**: Compile-time code generation with zero runtime overhead
- **Integration**: Works with existing error libraries like `thiserror`
- **Localization**: Enables structured error messages with argument extraction

## Usage Patterns

### Basic Enum with Custom Metadata
```rust
use errmeta_derive::ErrorMeta;

#[derive(Debug, ErrorMeta)]
enum MyError {
    #[error_meta(error_type = "ValidationError", code = "invalid_input")]
    InvalidInput { field: String, value: String },
    
    #[error_meta(error_type = "NetworkError")]  
    NetworkTimeout,  // Uses default code: "my_error-network_timeout"
}
```

### Transparent Error Wrapping
```rust
#[derive(Debug, ErrorMeta)]
enum ServiceError {
    #[error(transparent)]
    DatabaseError(DatabaseError),  // Delegates error_type(), code(), args()
    
    #[error(transparent)]
    #[error_meta(args_delegate = false)]
    IoError(std::io::Error),  // Custom args() with error string
}
```

### Struct Error Types
```rust
#[derive(Debug, ErrorMeta)]
#[error_meta(error_type = "BusinessLogicError", code = "insufficient_funds")]
struct InsufficientFundsError {
    account_id: String,
    requested_amount: i64,
    available_amount: i64,
}
```

### Trait Implementation
```rust
trait AppError {
    fn error_type(&self) -> String;
    fn code(&self) -> String;
    fn args(&self) -> std::collections::HashMap<String, String>;
}

#[derive(Debug, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum MyError {
    #[error_meta(error_type = "UserError")]
    InvalidInput,
}
```

## Integration Points

### With Object Layer (`objs`)
- Used by error enums in the `objs` crate for centralized error handling
- Integrates with localization service for error message formatting
- Provides structured error data for HTTP status code mapping

### With Service Layer
- Service-specific error types derive `ErrorMeta` for consistent error reporting
- Business logic errors include relevant context via args extraction
- Integration with tracing and logging systems via structured metadata

### With Localization System
- Generated `code()` method provides localization keys
- `args()` method extracts structured data for message templating
- Support for complex expressions in error_type and code attributes

## Supported Attributes

### Enum-Level Attributes
- `#[error_meta(trait_to_impl = TraitName)]` - Implement specified trait instead of inherent methods

### Variant-Level Attributes
- `#[error_meta(error_type = "ErrorType")]` - Custom error type (string literal or expression)
- `#[error_meta(code = "error_code")]` - Custom error code (string literal or expression)
- `#[error_meta(args_delegate = false)]` - Disable argument delegation for transparent errors

### Struct-Level Attributes
- `#[error_meta(trait_to_impl = TraitName)]` - Implement specified trait
- `#[error_meta(error_type = "ErrorType")]` - Required error type specification
- `#[error_meta(code = "error_code")]` - Optional error code (defaults to snake_case struct name)

## Field Pattern Support

### Named Fields
```rust
#[error_meta(error_type = "ValidationError")]
ValidationFailed { field_name: String, expected: String }
// Generates: args["field_name"] = field_name.to_string()
//           args["expected"] = expected.to_string()
```

### Unnamed Fields  
```rust
#[error_meta(error_type = "ParseError")]
ParseFailed(String, usize)
// Generates: args["var_0"] = var_0.to_string()
//           args["var_1"] = var_1.to_string()
```

### Unit Variants
```rust
#[error_meta(error_type = "SystemError")]
OutOfMemory
// Generates: empty HashMap
```

## Code Generation Features

### Automatic Code Generation
- Snake_case conversion: `MyError::InvalidInput` → `"my_error-invalid_input"`
- Type name conversion: `ValidationError` → `"validation_error"`
- Preserves custom codes when specified

### Expression Support
- Function calls: `#[error_meta(error_type = get_error_type())]`
- Enum variants: `#[error_meta(error_type = ErrorType::Validation)]`
- Complex expressions: `#[error_meta(code = self.generate_code())]`

### Transparent Error Handling
- Automatic delegation to wrapped error's metadata methods
- Optional argument delegation control via `args_delegate`
- Support for mixed transparent and non-transparent variants

## Testing and Validation

### Compile-Time Validation
- Missing required attributes trigger compile errors
- Invalid attribute syntax caught during macro expansion
- Type checking for expressions in error_meta attributes

### Runtime Testing
- Comprehensive test suite with `rstest` parameterized tests
- Integration tests with `thiserror` and other error libraries
- Trybuild tests for compile-time error validation

## Development Guidelines

### Adding New Attributes
1. Extend relevant `Parse` implementations for new attribute syntax
2. Update parsing functions to handle new attribute types
3. Modify code generation to incorporate new functionality
4. Add comprehensive tests for new features

### Error Handling Best Practices
- Use meaningful error codes for localization keys
- Include relevant context in error arguments
- Prefer transparent wrapping for external errors
- Test both compilation and runtime behavior

### Macro Development
- Use `cargo expand` to debug generated code
- Test edge cases with different field patterns
- Validate integration with existing error handling patterns
- Ensure generated code follows Rust conventions