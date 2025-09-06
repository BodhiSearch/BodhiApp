# Error System Refactoring Implementation Plan

## Overview

This plan documents the research and analysis for refactoring BodhiApp's complex error handling system. The current system uses a custom `errmeta_derive` procedural macro to generate error metadata for internationalization (i18n) and HTTP status mapping. The goal is to simplify this system while maintaining all functionality including error localization, HTTP status codes, and OpenAI API compatibility.

## Background and Motivation

### Current System Problems

1. **Complex Custom Macro System**: The `errmeta_derive` procedural macro adds compilation overhead and makes debugging difficult
2. **Rigid Type Mapping**: Single `ErrorType` per enum when variants need different HTTP status codes
3. **Tight Coupling**: Error definitions are tightly coupled with i18n, HTTP status mapping, and formatting logic
4. **Excessive Boilerplate**: Complex `From` implementations and `impl_error_from!` macro usage for cross-layer conversions
5. **Limited Flexibility**: Hard to attach contextual information dynamically to errors

### Current Implementation Details

The system consists of:
- **Custom derive macro** (`crates/errmeta_derive/`) generating `AppError` trait implementations
- **Fluent localization** for multi-language error messages
- **Error type mapping** to HTTP status codes
- **Cross-layer conversions** between objs → services → routes layers

## Architecture Analysis

### Core Components

#### 1. Error Metadata Derive Macro (`crates/errmeta_derive/src/lib.rs`)
- Generates implementations for `error_type()`, `code()`, `args()`, and `status()` methods
- Supports both enums and structs
- Handles transparent error delegation with `#[error(transparent)]`

#### 2. AppError Trait (`crates/objs/src/error/common.rs`)
```rust
pub trait AppError: std::error::Error + Send + Sync + 'static {
    fn error_type(&self) -> String;
    fn status(&self) -> u16;
    fn code(&self) -> String;
    fn args(&self) -> HashMap<String, String>;
}
```

#### 3. Localization Service (`crates/objs/src/localization_service.rs`)
- Singleton pattern with `FluentLocalizationService`
- Runtime message lookup with template interpolation
- Support for en-US and fr-FR locales

#### 4. Error Type Enum (`crates/objs/src/error/common.rs`)
- Maps error categories to HTTP status codes
- Used throughout the application for consistent status mapping

### Current Error Pattern Examples

#### Service Layer (`crates/services/src/hub_service.rs`)
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("hub_file_missing")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound)]
pub struct HubFileNotFoundError {
    pub filename: String,
    pub repo: String,
    pub snapshot: String,
}
```

#### Object Layer (`crates/objs/src/error/objs.rs`)
- Contains 20+ error types including IO errors, serialization errors, validation errors
- Each error type requires `ErrorMeta` derive and trait implementation

### Cross-Layer Conversion Patterns

#### impl_error_from Macro (`crates/objs/src/lib.rs`)
```rust
macro_rules! impl_error_from {
    ($source_error:ty, $target_error:ident :: $variant:ident, $intermediate_error:ty) => {
        impl From<$source_error> for $target_error {
            fn from(err: $source_error) -> Self {
                $target_error::$variant(<$intermediate_error>::from(err))
            }
        }
    }
}
```

## Proposed Solutions Analysis

### Solution 1: Simple thiserror + anyhow + Interceptor (Rejected)
**Concept**: Use simple error structs with serialization, handle all metadata in HTTP interceptor
**Why Rejected**: Still recreates what `errmeta_derive` does, just moved to different location

### Solution 2: Convention-Based Approach with Middleware (Most Promising)
**Concept**: 
- Errors as simple data structures with `thiserror` and `Serialize`
- No custom traits or metadata methods
- HTTP middleware deserializes errors and applies conventions for status/i18n
- Use `anyhow` for flexible context attachment

**Key Innovation**: Errors know nothing about HTTP or i18n - complete separation of concerns

## Detailed Design: Convention-Based Solution

### Core Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    objs     │────▶│  services   │────▶│   routes    │
│             │     │             │     │             │
│ Simple      │     │ anyhow::    │     │ anyhow::    │
│ Error       │     │ Result      │     │ Result      │
│ Structs     │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
                                    ┌──────────────────┐
                                    │   HTTP Layer     │
                                    │ Serialize errors │
                                    └──────────────────┘
                                               │
                                               ▼
                                    ┌──────────────────┐
                                    │  Interceptor     │
                                    │ • Deserialize    │
                                    │ • Determine      │
                                    │   status         │
                                    │ • Localize       │
                                    └──────────────────┘
```

### Implementation Components

#### 1. Simple Error Definitions
```rust
// No traits, no metadata, just data
#[derive(Debug, thiserror::Error, Serialize)]
#[error("hub_file_not_found")]  // This IS the i18n key
struct HubFileNotFound {
    path: String,
    repo: String,
}
```

#### 2. Convention-Based Status Mapping
```rust
fn determine_status(error_type: &str) -> StatusCode {
    if error_type.contains("not_found") { StatusCode::NOT_FOUND }
    else if error_type.contains("unauthorized") { StatusCode::UNAUTHORIZED }
    else if error_type.contains("forbidden") { StatusCode::FORBIDDEN }
    else if error_type.contains("bad_request") { StatusCode::BAD_REQUEST }
    else if error_type.contains("unavailable") { StatusCode::SERVICE_UNAVAILABLE }
    else { StatusCode::INTERNAL_SERVER_ERROR }
}
```

#### 3. Axum Middleware for Translation
**Location**: `crates/server_core/src/middleware/i18n.rs`
- Extract Accept-Language header
- Deserialize error from response body
- Apply conventions for status code
- Localize message using Fluent
- Rebuild OpenAI-compatible response

## Implementation Strategy

### Phase 1: Foundation (Week 1)
1. Create middleware infrastructure in `crates/server_core/src/middleware/`
2. Implement convention-based status determination
3. Set up error serialization/deserialization pipeline
4. Create test harness for middleware

### Phase 2: Pilot Migration (Week 2)
1. Migrate `crates/services/src/hub_service.rs` errors
2. Remove `ErrorMeta` derives from hub errors
3. Convert to simple structs with `Serialize`
4. Validate i18n still works through middleware
5. Benchmark compilation time improvements

### Phase 3: Object Layer Migration (Week 3)
1. Migrate common errors in `crates/objs/src/error/objs.rs`
2. Remove `AppError` trait implementations
3. Update error definitions to simple structs
4. Ensure serialization captures all needed fields

### Phase 4: Service Layer Migration (Week 3-4)
1. Convert all service errors to `anyhow::Result`
2. Remove `impl_error_from!` macro usage
3. Add context using `.context()` where appropriate
4. Update error propagation patterns

### Phase 5: Routes Layer Integration (Week 4)
1. Update route handlers to use `anyhow::Result`
2. Ensure errors serialize properly to HTTP responses
3. Test middleware intercepts and translates correctly
4. Validate OpenAI API compatibility maintained

### Phase 6: Cleanup (Week 5)
1. Delete `crates/errmeta_derive/` entirely
2. Remove `AppError` trait from `crates/objs/`
3. Clean up `impl_error_from!` macro
4. Update documentation

## Key Design Decisions

### 1. Use Serialization as Contract
**Decision**: Errors serialize to JSON, which becomes the contract
**Rationale**: Eliminates need for custom traits while preserving all data
**Trade-off**: Requires middleware to deserialize, but centralizes complexity

### 2. Convention Over Configuration
**Decision**: Determine HTTP status from error name patterns
**Rationale**: Reduces boilerplate, follows REST conventions
**Trade-off**: Less explicit but more maintainable

### 3. Complete Separation of Concerns
**Decision**: Errors know nothing about HTTP or i18n
**Rationale**: Better separation, easier testing, cleaner code
**Trade-off**: All intelligence moves to middleware layer

### 4. Leverage anyhow for Context
**Decision**: Use `anyhow::Result` throughout services/routes
**Rationale**: Industry standard, great ergonomics, no From implementations needed
**Trade-off**: Less type safety but more flexibility

## Testing Strategy

### Unit Tests
1. Test error serialization produces expected JSON structure
2. Test convention-based status determination
3. Test middleware error interception and translation
4. Test anyhow context propagation

### Integration Tests
1. End-to-end error flow from service to HTTP response
2. Localization with different Accept-Language headers
3. OpenAI API error format compatibility
4. Error context preservation through layers

### Performance Tests
1. Benchmark compilation time before/after removing proc macro
2. Measure runtime overhead of middleware interception
3. Profile memory usage of error serialization

## Migration Path

### Gradual Migration Strategy
1. Keep old system working during transition
2. Use feature flags to switch between old/new per module
3. Migrate one service at a time
4. Run both systems in parallel initially
5. Remove old system only after full validation

### Rollback Plan
1. Git branches for each migration phase
2. Feature flags to revert to old system
3. Keep errmeta_derive until fully validated
4. Document rollback procedures

## Success Criteria

- ✅ 30-50% reduction in compilation time
- ✅ All existing error messages preserved
- ✅ I18n continues working with Fluent
- ✅ OpenAI API compatibility maintained
- ✅ No custom derive macros needed
- ✅ Simplified error definitions
- ✅ Clean separation between errors and HTTP/i18n concerns
- ✅ Standard error handling patterns (thiserror + anyhow)

## Challenges and Considerations

### Technical Challenges
1. **Middleware Complexity**: All error intelligence concentrated in middleware
2. **Serialization Overhead**: Errors must be serialized/deserialized
3. **Convention Ambiguity**: Some errors may not fit naming conventions
4. **Testing Complexity**: Need comprehensive middleware testing

### Migration Risks
1. **Breaking Changes**: Error format changes could break clients
2. **Performance Impact**: Serialization overhead needs measurement
3. **Hidden Dependencies**: Some code may depend on AppError trait
4. **Localization Gaps**: Ensure all error messages have translations

## Alternative Approaches Considered

### 1. Pure thiserror with Manual Metadata
- Keep implementing traits manually
- Rejected: Just moves boilerplate, doesn't reduce complexity

### 2. error-stack Library
- Use error-stack for rich context
- Rejected: Overkill for our needs, adds heavy dependency

### 3. miette for Diagnostics
- Use miette's diagnostic system
- Rejected: Designed for CLI, not web APIs

### 4. Custom Error Registry
- Central registry mapping error types to metadata
- Rejected: Still requires registration boilerplate

## Future Enhancements

1. **Dynamic Status Override**: Allow explicit status override when needed
2. **Error Telemetry**: Add error tracking and metrics
3. **Client-Side i18n**: Support client-side message translation
4. **Error Recovery Hints**: Add suggested actions to error responses
5. **GraphQL Support**: Extend pattern to GraphQL errors

## Key Files and Methods

### Current System Files
- `crates/errmeta_derive/src/lib.rs` - Procedural macro implementation
- `crates/objs/src/error/common.rs` - AppError trait and ErrorType enum
- `crates/objs/src/error/objs.rs` - Common error definitions
- `crates/objs/src/localization_service.rs` - Fluent localization service
- `crates/objs/src/error/error_api.rs` - API error conversion

### Files to Create
- `crates/server_core/src/middleware/i18n.rs` - Error translation middleware
- `crates/server_core/src/middleware/error_interceptor.rs` - Error interception logic

### Files to Modify (Examples)
- `crates/services/src/hub_service.rs` - Convert to simple errors
- `crates/services/src/data_service.rs` - Use anyhow::Result
- `crates/routes_app/src/error.rs` - Remove ErrorMeta derives

## References

### Research Resources
- thiserror documentation: https://docs.rs/thiserror
- anyhow documentation: https://docs.rs/anyhow
- Axum middleware patterns: https://docs.rs/axum/latest/axum/middleware
- Fluent localization: https://projectfluent.org/

### Related Discussions
- Initial error system design decisions
- I18n requirements and constraints
- OpenAI API compatibility requirements
- HTTP status code mapping patterns

## Conclusion

This refactoring represents a fundamental architectural shift from compile-time metadata generation to runtime convention-based handling. While complex to implement, it promises significant improvements in compilation time, code simplicity, and maintainability. The key insight is separating "what went wrong" (the error) from "how to present it" (HTTP/i18n concerns), leading to cleaner, more maintainable code.

The plan should be executed gradually with careful testing at each phase to ensure no functionality is lost while gaining the benefits of a simpler, more flexible error handling system.