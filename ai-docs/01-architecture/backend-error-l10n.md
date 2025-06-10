# Backend Error Handling and Internationalization System

## Overview

BodhiApp implements a sophisticated error handling and internationalization (i18n) system that combines HTTP status codes, internationalized error messages using Mozilla's Fluent framework, and a custom error macro for consistent error handling across the multi-project Rust workspace. This system ensures that all errors provide both machine-readable metadata and human-readable, localized messages.

## Architectural Rationale

The system addresses several key requirements:

1. **Consistent Error Format**: All errors across different crates provide the same metadata structure
2. **Internationalization**: Error messages can be localized using Fluent's powerful message formatting
3. **HTTP Integration**: Errors automatically map to appropriate HTTP status codes
4. **Developer Experience**: Rich error information for debugging with minimal boilerplate
5. **Type Safety**: Compile-time guarantees for error metadata completeness

## Core Components

### 1. Localization Service Singleton

**File**: `crates/objs/src/localization_service.rs:70-100`

The `FluentLocalizationService` implements a singleton pattern for global error message access:

```rust
static INSTANCE: Lazy<Mutex<Option<Arc<FluentLocalizationService>>>> = 
    Lazy::new(|| Mutex::new(None));

impl FluentLocalizationService {
    pub fn get_instance() -> Arc<FluentLocalizationService> {
        // Thread-safe singleton initialization
    }
}
```

**Key Features**:
- Thread-safe singleton using `once_cell::sync::Lazy`
- Concurrent message lookup with `RwLock<HashMap<LanguageIdentifier, FluentBundle>>`
- Support for multiple locales (currently en-US and fr-FR)
- Resource loading from embedded directories

### 2. AppError Trait

**File**: `crates/objs/src/error.rs:50-61`

The `AppError` trait defines the contract for all application errors:

```rust
pub trait AppError: std::error::Error {
    fn error_type(&self) -> String;
    fn status(&self) -> u16;
    fn code(&self) -> String;
    fn args(&self) -> HashMap<String, String>;
}
```

**Responsibilities**:
- `error_type()`: Returns error category (validation_error, internal_server_error, etc.)
- `status()`: HTTP status code derived from error type
- `code()`: Unique error code for message lookup
- `args()`: Parameters for message interpolation

### 3. ErrorMeta Derive Macro

**File**: `crates/errmeta_derive/src/lib.rs:10-15`

The custom `ErrorMeta` procedural macro automatically implements the `AppError` trait:

```rust
#[proc_macro_derive(ErrorMeta, attributes(error_meta))]
pub fn derive_error_metadata(input: TokenStream) -> TokenStream
```

**Capabilities**:
- Automatic code generation for error metadata
- Support for both enums and structs
- Transparent error delegation
- Default code generation using snake_case conversion
- Compile-time validation of required attributes

**Usage Example**:
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LoginError {
    #[error("app_reg_info_not_found")]
    #[error_meta(error_type = ErrorType::InvalidAppState)]
    AppRegInfoNotFound,
    
    #[error(transparent)]
    SecretServiceError(#[from] SecretServiceError),
}
```

### 4. ApiError Conversion

**File**: `crates/objs/src/error_api.rs:35-46`

The `ApiError` struct provides a consistent JSON format for HTTP responses:

```rust
impl<T: AppError + 'static> From<T> for ApiError {
    fn from(value: T) -> Self {
        ApiError {
            name: value.to_string(),
            error_type: value.error_type(),
            status: value.status(),
            code: value.code(),
            args: value.args(),
        }
    }
}
```

## Localization Infrastructure

### 1. Resource Embedding

**Pattern**: Each crate defines an `l10n` module with embedded resources:

```rust
// crates/objs/src/lib.rs:48-52
pub mod l10n {
    use include_dir::Dir;
    pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
```

### 2. Message File Structure

**Location**: `src/resources/en-US/messages.ftl` in each crate

**Naming Convention**: `{error_enum_snake_case}-{variant_snake_case}`

**Examples**:
- `login_error-app_reg_info_not_found = app is not registered, need to register app first`
- `io_error = io_error: {$source}`
- `validation_errors = {$var_0}`

### 3. Message Interpolation

Fluent supports rich message formatting with parameters:

```ftl
hub_service_error-gated_access = {$source}.
  huggingface repo '{$repo}' is requires requesting for access from website.
  Go to https://huggingface.co/{$repo} to request access to the model and try again.
```

## Initialization Flow

### 1. Application Startup

**File**: `crates/bodhi/src-tauri/src/app.rs:95-108`

During application initialization, all crate resources are registered:

```rust
let localization_service = FluentLocalizationService::get_instance();
localization_service
    .load_resource(objs::l10n::L10N_RESOURCES)?
    .load_resource(objs::gguf::l10n::L10N_RESOURCES)?
    .load_resource(llama_server_proc::l10n::L10N_RESOURCES)?
    // ... all other crates
```

### 2. Test Environment

**File**: `crates/objs/src/test_utils/l10n.rs:39-95`

Test utilities provide mock localization service setup:

```rust
#[fixture]
#[once]
pub fn setup_l10n(localization_service: Arc<FluentLocalizationService>) -> Arc<FluentLocalizationService> {
    // Load all crate resources for testing
    set_mock_localization_service(localization_service.clone());
    localization_service
}
```

## Project-Specific Error Organization

### 1. Error Consolidation Pattern

Each crate maintains a central `src/error.rs` file that consolidates all project-specific errors:

**Example**: `crates/routes_app/src/error.rs:5-34`

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LoginError {
    #[error("app_reg_info_not_found")]
    #[error_meta(error_type = ErrorType::InvalidAppState)]
    AppRegInfoNotFound,
    
    #[error(transparent)]
    SecretServiceError(#[from] SecretServiceError),
}
```

### 2. Error Type Mapping

**File**: `crates/objs/src/error.rs:13-33`

Error types map to HTTP status codes:

```rust
pub enum ErrorType {
    Validation,           // 400
    BadRequest,          // 400
    InvalidAppState,     // 400
    InternalServer,      // 500
    Authentication,      // 401
    Forbidden,          // 403
    NotFound,           // 404
    Unknown,            // 500
}
```

## Setup Guide for New Projects

### 1. Add Dependencies

```toml
[dependencies]
errmeta_derive = { workspace = true }
thiserror = { workspace = true }
objs = { workspace = true }
include_dir = { workspace = true }
```

### 2. Create Error Module

Create `src/error.rs`:

```rust
use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MyProjectError {
    #[error("operation_failed")]
    #[error_meta(error_type = ErrorType::InternalServer)]
    OperationFailed(String),
    
    #[error(transparent)]
    ExternalError(#[from] SomeExternalError),
}
```

### 3. Create Message Files

Create `src/resources/en-US/messages.ftl`:

```ftl
my_project_error-operation_failed = operation failed: {$var_0}
```

### 4. Add L10n Module

In `src/lib.rs`:

```rust
pub mod l10n {
    use include_dir::Dir;
    pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
```

### 5. Register Resources

Add to application initialization:

```rust
localization_service.load_resource(my_project::l10n::L10N_RESOURCES)?;
```

## Error Handling Patterns

### 1. Transparent Error Delegation

For errors that should pass through unchanged:

```rust
#[error(transparent)]
DatabaseError(#[from] sqlx::Error),
```

### 2. Custom Error Codes

For specific error identification:

```rust
#[error("custom_operation_failed")]
#[error_meta(error_type = ErrorType::BadRequest, code = "CUSTOM_OP_FAIL")]
CustomOperationFailed,
```

### 3. Argument Delegation Control

To prevent automatic argument extraction:

```rust
#[error(transparent)]
#[error_meta(args_delegate = false)]
ExternalError(#[from] ExternalLibError),
```

## Integration with Application Architecture

### 1. HTTP Response Generation

Errors automatically convert to consistent JSON responses via `ApiError`:

```json
{
  "error": {
    "message": "app is not registered, need to register app first",
    "type": "invalid_app_state",
    "code": "login_error-app_reg_info_not_found"
  }
}
```

### 2. Logging Integration

Error metadata enhances structured logging:

```rust
tracing::error!(
    error_type = %error.error_type(),
    error_code = %error.code(),
    "Operation failed"
);
```

### 3. Testing Support

Test utilities provide error message validation:

```rust
#[rstest]
fn test_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
}
```

## Design Decisions and Trade-offs

### 1. Singleton Pattern Choice

**Rationale**: Global access to localization service without dependency injection complexity
**Trade-off**: Reduced testability, addressed through test-specific mock implementation

### 2. Compile-time Resource Embedding

**Rationale**: Zero-runtime overhead for resource loading, simplified deployment
**Trade-off**: Larger binary size, rebuild required for message changes

### 3. Fluent Over Simple Key-Value

**Rationale**: Rich formatting capabilities, proper pluralization, gender support
**Trade-off**: Additional complexity over simple string interpolation

### 4. Procedural Macro Approach

**Rationale**: Eliminates boilerplate while maintaining type safety
**Trade-off**: Compile-time complexity, debugging challenges for macro-generated code

## Comparison with thiserror

The `errmeta_derive` macro extends `thiserror` capabilities:

**thiserror provides**:
- Error trait implementation
- Display formatting
- Source error chaining

**errmeta_derive adds**:
- HTTP status code mapping
- Localization code generation
- Structured error metadata
- Automatic argument extraction
- API response formatting

**Usage Pattern**:
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("database_connection_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct DatabaseError {
    #[from]
    source: sqlx::Error,
    database_url: String,
}
```

This generates both `thiserror`'s standard error implementation and `errmeta_derive`'s metadata methods, providing a complete error handling solution that integrates seamlessly with the localization system.

## Advanced Features

### 1. Error Code Generation

**File**: `crates/errmeta_derive/src/lib.rs:382-391`

Default error codes follow the pattern `{enum_name}-{variant_name}` in snake_case:

```rust
// LoginError::AppRegInfoNotFound becomes "login_error-app_reg_info_not_found"
let default_code = format!(
    "{}-{}",
    name.to_string().to_case(Case::Snake),
    variant_name.to_string().to_case(Case::Snake)
);
```

### 2. Transparent Error Handling

**File**: `crates/errmeta_derive/src/lib.rs:350-360`

Transparent variants delegate all metadata to the wrapped error:

```rust
if is_transparent {
    quote! {
        #name::#variant_name(err) => err.error_type(),
    }
}
```

### 3. Concurrent Message Access

**File**: `crates/objs/src/localization_service.rs:202-246`

The localization service supports concurrent access through `RwLock`:

```rust
let bundles = self.bundles.read().map_err(|err| RwLockReadError::new(err.to_string()))?;
let bundle = bundles.get(locale).ok_or_else(|| LocaleNotSupportedError::new(code.to_string()))?;
```

This error handling and internationalization system provides a robust foundation for consistent, user-friendly error reporting across the entire BodhiApp ecosystem while maintaining developer productivity and type safety.
