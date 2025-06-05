# errmeta_derive - Error Metadata Derive Macros

## Overview

The `errmeta_derive` crate provides procedural macros for generating error metadata in BodhiApp. It automates the creation of rich error information including localization keys, error codes, and structured error data that enhances debugging and user experience.

## Purpose

- **Error Metadata Generation**: Automatic generation of error metadata from error types
- **Localization Support**: Generate localization keys for error messages
- **Error Code Assignment**: Automatic error code generation and management
- **Debugging Enhancement**: Rich error information for debugging and troubleshooting
- **API Documentation**: Generate error documentation for OpenAPI specifications

## Key Features

### Derive Macro

#### `#[derive(ErrorMeta)]`
The primary derive macro that generates error metadata for error enums:

```rust
use errmeta_derive::ErrorMeta;

#[derive(ErrorMeta, Debug, thiserror::Error)]
pub enum MyError {
    #[error("Invalid input: {field}")]
    #[error_code("INVALID_INPUT")]
    #[error_l10n("errors.invalid_input")]
    InvalidInput { field: String },
    
    #[error("Resource not found")]
    #[error_code("NOT_FOUND")]
    #[error_l10n("errors.not_found")]
    NotFound,
    
    #[error("Internal error: {source}")]
    #[error_code("INTERNAL_ERROR")]
    #[error_l10n("errors.internal")]
    Internal { source: Box<dyn std::error::Error> },
}
```

### Generated Functionality

The derive macro generates several implementations:

#### Error Code Implementation
```rust
impl ErrorCode for MyError {
    fn error_code(&self) -> &'static str {
        match self {
            MyError::InvalidInput { .. } => "INVALID_INPUT",
            MyError::NotFound => "NOT_FOUND",
            MyError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}
```

#### Localization Key Implementation
```rust
impl LocalizationKey for MyError {
    fn l10n_key(&self) -> &'static str {
        match self {
            MyError::InvalidInput { .. } => "errors.invalid_input",
            MyError::NotFound => "errors.not_found",
            MyError::Internal { .. } => "errors.internal",
        }
    }
}
```

#### Error Metadata Implementation
```rust
impl ErrorMetadata for MyError {
    fn metadata(&self) -> ErrorMeta {
        ErrorMeta {
            code: self.error_code(),
            l10n_key: self.l10n_key(),
            severity: self.severity(),
            category: self.category(),
            details: self.details(),
        }
    }
}
```

## Directory Structure

```
src/
└── lib.rs                    # Main procedural macro implementation
```

## Macro Attributes

### Error Code Attributes

#### `#[error_code("CODE")]`
Specifies the error code for the variant:
```rust
#[error_code("VALIDATION_FAILED")]
ValidationError { field: String },
```

#### `#[error_code_prefix("PREFIX")]`
Sets a prefix for all error codes in the enum:
```rust
#[derive(ErrorMeta)]
#[error_code_prefix("AUTH")]
pub enum AuthError {
    #[error_code("INVALID_TOKEN")]  // Results in "AUTH_INVALID_TOKEN"
    InvalidToken,
}
```

### Localization Attributes

#### `#[error_l10n("key")]`
Specifies the localization key for the variant:
```rust
#[error_l10n("errors.auth.invalid_credentials")]
InvalidCredentials,
```

#### `#[error_l10n_prefix("prefix")]`
Sets a prefix for all localization keys:
```rust
#[derive(ErrorMeta)]
#[error_l10n_prefix("errors.database")]
pub enum DatabaseError {
    #[error_l10n("connection_failed")]  // Results in "errors.database.connection_failed"
    ConnectionFailed,
}
```

### Severity Attributes

#### `#[error_severity(level)]`
Specifies the error severity:
```rust
#[error_severity(Critical)]
SystemFailure,

#[error_severity(Warning)]
DeprecatedFeature,

#[error_severity(Info)]
ConfigurationChange,
```

### Category Attributes

#### `#[error_category("category")]`
Specifies the error category:
```rust
#[error_category("validation")]
InvalidInput { field: String },

#[error_category("authentication")]
Unauthorized,

#[error_category("system")]
InternalError,
```

## Generated Traits

### ErrorCode Trait
```rust
pub trait ErrorCode {
    fn error_code(&self) -> &'static str;
}
```

### LocalizationKey Trait
```rust
pub trait LocalizationKey {
    fn l10n_key(&self) -> &'static str;
}
```

### ErrorMetadata Trait
```rust
pub trait ErrorMetadata {
    fn metadata(&self) -> ErrorMeta;
    fn severity(&self) -> ErrorSeverity;
    fn category(&self) -> &'static str;
    fn details(&self) -> Option<serde_json::Value>;
}
```

## Error Metadata Structure

### ErrorMeta Struct
```rust
pub struct ErrorMeta {
    pub code: &'static str,
    pub l10n_key: &'static str,
    pub severity: ErrorSeverity,
    pub category: &'static str,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
}
```

### ErrorSeverity Enum
```rust
pub enum ErrorSeverity {
    Critical,    // System-breaking errors
    Error,       // Standard errors
    Warning,     // Non-critical issues
    Info,        // Informational messages
}
```

## Usage Examples

### Basic Error Definition
```rust
use errmeta_derive::ErrorMeta;

#[derive(ErrorMeta, Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Validation failed for field: {field}")]
    #[error_code("VALIDATION_FAILED")]
    #[error_l10n("api.validation.failed")]
    #[error_severity(Error)]
    #[error_category("validation")]
    ValidationFailed { field: String },
    
    #[error("User not authorized")]
    #[error_code("UNAUTHORIZED")]
    #[error_l10n("api.auth.unauthorized")]
    #[error_severity(Warning)]
    #[error_category("authentication")]
    Unauthorized,
}
```

### Nested Error Handling
```rust
#[derive(ErrorMeta, Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {source}")]
    #[error_code("DATABASE_ERROR")]
    #[error_l10n("service.database.error")]
    Database {
        #[from]
        source: DatabaseError,
    },
    
    #[error("External API error: {source}")]
    #[error_code("EXTERNAL_API_ERROR")]
    #[error_l10n("service.external.error")]
    ExternalApi {
        #[from]
        source: reqwest::Error,
    },
}
```

### Error Usage
```rust
fn validate_input(input: &str) -> Result<(), ApiError> {
    if input.is_empty() {
        return Err(ApiError::ValidationFailed {
            field: "input".to_string(),
        });
    }
    Ok(())
}

// Error metadata usage
match validate_input("") {
    Err(error) => {
        let metadata = error.metadata();
        println!("Error code: {}", metadata.code);
        println!("Localization key: {}", metadata.l10n_key);
        println!("Severity: {:?}", metadata.severity);
    }
    Ok(_) => println!("Success"),
}
```

## Integration with BodhiApp

### API Error Responses
The generated metadata integrates with HTTP error responses:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let metadata = self.metadata();
        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: metadata.code,
                message: self.to_string(),
                l10n_key: metadata.l10n_key,
                severity: metadata.severity,
                category: metadata.category,
                details: metadata.details,
                timestamp: metadata.timestamp,
            },
        };
        
        let status = match metadata.severity {
            ErrorSeverity::Critical => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorSeverity::Error => StatusCode::BAD_REQUEST,
            ErrorSeverity::Warning => StatusCode::BAD_REQUEST,
            ErrorSeverity::Info => StatusCode::OK,
        };
        
        (status, Json(error_response)).into_response()
    }
}
```

### Localization Integration
```rust
pub fn localize_error(error: &dyn ErrorMetadata, locale: &str) -> String {
    let metadata = error.metadata();
    let localization_service = get_localization_service();
    localization_service.get_message(metadata.l10n_key, locale)
        .unwrap_or_else(|| error.to_string())
}
```

## Dependencies

### Core Dependencies
- **proc-macro2**: Procedural macro utilities
- **quote**: Code generation utilities
- **syn**: Rust syntax parsing

### Generated Code Dependencies
- **serde**: Serialization for error details
- **chrono**: Timestamp generation
- **thiserror**: Error trait implementation

## Compilation

The derive macro is a procedural macro that runs at compile time:

```toml
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

## Testing Support

### Macro Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_metadata_generation() {
        let error = MyError::InvalidInput {
            field: "username".to_string(),
        };
        
        assert_eq!(error.error_code(), "INVALID_INPUT");
        assert_eq!(error.l10n_key(), "errors.invalid_input");
        assert_eq!(error.metadata().category, "validation");
    }
}
```

### Integration Testing
The macro integrates with the broader error handling system:

```rust
#[test]
fn test_error_response_generation() {
    let error = ApiError::ValidationFailed {
        field: "email".to_string(),
    };
    
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

## Future Extensions

The errmeta_derive crate is designed to support:
- **Custom Metadata Fields**: Additional metadata fields for specific use cases
- **Error Aggregation**: Combine multiple errors with shared metadata
- **Error Transformation**: Transform errors between different types
- **Advanced Localization**: Context-aware localization with parameters
- **Error Analytics**: Generate error analytics and reporting data
