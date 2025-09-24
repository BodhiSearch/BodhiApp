# CLAUDE.md - errmeta_derive

See [crates/errmeta_derive/PACKAGE.md](crates/errmeta_derive/PACKAGE.md) for implementation details and file references.

## Architectural Purpose

The `errmeta_derive` crate serves as BodhiApp's foundational procedural macro infrastructure for structured error handling. It transforms Rust error enums and structs into self-describing error objects that provide consistent metadata extraction across all application layers. This macro-driven approach ensures zero runtime overhead while enabling sophisticated error categorization, localization, and debugging capabilities throughout the system.

## Domain Architecture

### Error Metadata Extraction System

The crate implements a three-method contract for error introspection:

- **Error Type Classification**: `error_type()` returns semantic error categories (ValidationError, DatabaseError, etc.) that enable service-level error routing and HTTP status code mapping
- **Localization Key Generation**: `code()` provides stable identifiers for error message templates, supporting multi-language user interfaces
- **Structured Argument Extraction**: `args()` captures error context as key-value pairs for message interpolation and debugging

This triumvirate approach separates concerns between error semantics, user presentation, and diagnostic information, enabling clean error handling patterns across service boundaries.

### Compile-Time Code Generation Architecture

The macro operates through a sophisticated token stream transformation pipeline:

1. **Syntax Tree Analysis**: Parses Rust AST to identify error structure patterns and validate attribute specifications
2. **Attribute Expression Evaluation**: Supports arbitrary Rust expressions in error metadata attributes, enabling dynamic error categorization
3. **Pattern Matching Generation**: Creates exhaustive match statements that handle all field combinations while maintaining type safety
4. **Trait Implementation Flexibility**: Supports both inherent method implementation and configurable trait implementation via `trait_to_impl`

This design ensures that error metadata extraction integrates seamlessly with existing Rust error handling patterns while providing extensibility for future requirements.

### Transparent Error Delegation Framework

The crate's transparent error support enables error chain preservation while maintaining metadata consistency:

- **Automatic Delegation**: `#[error(transparent)]` variants automatically forward metadata calls to wrapped errors
- **Selective Override Capability**: Transparent variants can override `error_type` and `code` while preserving argument delegation
- **Configurable Argument Handling**: `args_delegate = false` provides fine-grained control over argument extraction for wrapped errors

This framework is crucial for BodhiApp's multi-layered architecture, where errors propagate through service boundaries while preserving both the original error context and service-specific metadata.

## Cross-Crate Integration Patterns

### Foundation for Object Layer Error Handling

The `objs` crate relies on `errmeta_derive` to implement the `AppError` trait across all domain error types. This integration provides:

- **Consistent HTTP Response Generation**: Error metadata drives status code selection and response body formatting
- **Localization Service Integration**: Error codes serve as keys for message template lookup and user language selection
- **Structured Logging Integration**: Error arguments provide contextual information for observability and debugging

### Service Layer Error Coordination

Service-specific error types derive `ErrorMeta` to ensure consistent error handling across business logic boundaries:

- **Error Propagation Patterns**: Transparent error wrapping enables clean error forwarding while preserving service-specific context
- **Tracing Integration**: Structured error metadata integrates with tracing spans for comprehensive request lifecycle visibility
- **Business Logic Error Context**: Named fields in error variants automatically become structured logging attributes

### API Layer Error Translation

Route handlers leverage generated error metadata to transform internal errors into API responses:

- **OpenAPI Documentation Generation**: Error metadata enables automatic API documentation of error response schemas
- **Client Error Consistency**: Generated error codes provide stable identifiers for client-side error handling
- **Debugging Information**: Structured arguments enable detailed error information in development environments while maintaining security in production

## Technical Constraints and Design Decisions

### Compile-Time Validation Requirements

The macro enforces strict compilation rules to ensure error handling consistency:

- **Mandatory Error Type Specification**: Non-transparent enum variants must specify `error_type` to prevent runtime classification errors
- **Expression Syntax Validation**: Attribute expressions undergo full Rust syntax validation during macro expansion
- **Union Type Prohibition**: Union types are explicitly rejected due to undefined field access patterns

These constraints prevent common error handling mistakes and ensure that all error types provide complete metadata.

### Expression Evaluation Flexibility

The attribute system supports sophisticated error metadata customization:

- **Runtime Expression Evaluation**: Attributes can contain complex expressions that evaluate to strings at runtime
- **Context-Aware Error Generation**: Methods like `self.generate_code()` enable dynamic error codes based on error state
- **Type System Integration**: Enum variants as error types leverage Rust's type system for categorization consistency

This flexibility enables context-sensitive error handling while maintaining compile-time safety guarantees.

### Memory and Performance Architecture

The macro's design prioritizes zero-runtime-cost abstractions:

- **Compile-Time Code Generation**: All metadata extraction logic is generated at compile time, eliminating runtime reflection overhead
- **Efficient Pattern Matching**: Generated match statements compile to optimal machine code without dynamic dispatch
- **String Allocation Minimization**: Error codes and types use efficient string formatting with minimal allocations

This approach ensures that error handling remains performant even in high-throughput scenarios.

## Error Handling Evolution Patterns

### Extensibility Design

The macro architecture supports evolutionary error handling requirements:

- **Attribute Extension Points**: New attributes can be added without breaking existing error type definitions
- **Backward Compatibility**: Existing error types continue to function when new features are added to the macro
- **Migration Pathways**: Transparent error support enables gradual migration of error handling patterns across the codebase

### Integration with External Error Libraries

The macro integrates seamlessly with Rust's error handling ecosystem:

- **thiserror Integration**: Generated code works naturally with `#[derive(thiserror::Error)]` for comprehensive error handling
- **anyhow Compatibility**: Error metadata remains accessible when errors are wrapped in `anyhow::Error`
- **Standard Library Coordination**: Transparent error delegation works with standard library error types

## Domain-Specific Knowledge

### BodhiApp Error Categorization System

The error type classification aligns with BodhiApp's service architecture:

- **Domain Error Types**: ValidationError, AuthenticationError, AuthorizationError map to business logic concerns
- **Infrastructure Error Types**: DatabaseError, NetworkError, FileSystemError map to operational concerns
- **Integration Error Types**: ExternalServiceError, HubApiError map to external dependency failures

### Localization Architecture Integration

Error code generation follows BodhiApp's internationalization patterns:

- **Hierarchical Code Structure**: Error codes use `service_name-error_variant` format for namespace organization
- **Template Parameter Extraction**: Field names become template parameters for message interpolation
- **Cultural Context Preservation**: Error arguments preserve cultural formatting requirements for numbers, dates, and currencies

### Security and Privacy Considerations

The error metadata system balances debugging needs with security requirements:

- **Sensitive Information Filtering**: Transparent errors can override argument delegation to prevent information leakage
- **Production Error Sanitization**: Error codes provide stable identifiers while potentially sensitive error messages remain internal
- **Audit Trail Integration**: Structured error arguments support compliance requirements for error tracking and investigation