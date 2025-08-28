# PACKAGE.md - objs

This document serves as a rich index for the `objs` crate, guiding AI assistants to BodhiApp-specific implementation patterns and explaining why architectural decisions exist in the application's **universal foundation layer**.

## BodhiApp Domain Architecture Position

The `objs` crate serves as the **architectural keystone** for BodhiApp, providing foundational types used throughout the entire application ecosystem. Every other crate depends on `objs` for:

- **Cross-Crate Consistency**: Unified domain entities across services, routes, CLI, and desktop components
- **Error System Integration**: Centralized error handling with multi-language localization
- **External API Bridging**: Standardized interfaces for OpenAI, Hugging Face, and OAuth2 systems
- **Business Rule Enforcement**: Domain validation and type safety across all application layers

## Centralized Error System Implementation

BodhiApp's error architecture provides application-wide consistency through sophisticated code generation patterns:

```rust
// Pattern structure (see src/error/objs.rs:15-45 for complete implementations)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServiceError {
    #[error("model_not_found")]
    #[error_meta(error_type = ErrorType::NotFound)]
    ModelNotFound { model_name: String },
}
```

### **Cross-Crate Error Flow Architecture**
The error system coordinates across all BodhiApp layers through these key files:

- **Error Definition**: `src/error/objs.rs` - Contains all domain error types with `errmeta_derive` patterns
- **API Conversion**: `src/error/error_api.rs:45-89` - `ApiError` envelope for OpenAI compatibility
- **Localization**: `src/localization_service.rs:78-156` - Singleton service with test override mechanism
- **Resource Templates**: `src/resources/en-US/messages.ftl` - Fluent message templates with argument extraction

**Why This Architecture**: Enables seamless error propagation from services → routes → clients with consistent localization and OpenAI API compatibility across all application boundaries.

## GGUF Binary Format System

BodhiApp's comprehensive GGUF support enables safe local AI model management through specialized parsing:

```rust
// Core pattern (see src/gguf/metadata.rs:89-145 for complete implementation)
let metadata = GGUFMetadata::new(&path)?;
let chat_template = metadata.get_value("tokenizer.chat_template")?.as_str()?;
```

### **Service Integration Architecture**
GGUF parsing integrates throughout BodhiApp's model management pipeline via these implementations:

- **Core Parser**: `src/gguf/metadata.rs:45-234` - Memory-mapped file access with bounds checking
- **Value System**: `src/gguf/value.rs:23-156` - Typed accessors with endian autodetection
- **Error Handling**: `src/gguf/error.rs:15-67` - Localized diagnostics for file corruption
- **Constants**: `src/gguf/constants.rs` - Magic numbers and version validation

**Why This Architecture**: Prevents crashes on corrupted model files while providing detailed diagnostics. Cross-platform endian support enables model sharing between different architectures used by HubService, DataService, and CLI components.

## Model Ecosystem Management Architecture

BodhiApp's sophisticated model management system coordinates between external repositories and local storage through domain objects:

```rust
// Core patterns (see respective files for complete implementations)
impl TryFrom<PathBuf> for HubFile {  // src/hub_file.rs:67-89
    fn try_from(path: PathBuf) -> Result<Self, HFCachePathError> {
        // Validates HF cache structure: .../models--<user>--<name>/snapshots/<hash>/<file>
    }
}
```

### **Cross-Service Model Coordination**
Model entities enable complex service interactions through these key implementations:

- **Repository Format**: `src/repo.rs:45-78` - Enforces "user/name" format with validation
- **HubFile Management**: `src/hub_file.rs:23-156` - Hugging Face cache structure validation
- **Alias System**: `src/alias.rs:89-234` - YAML configuration with filename sanitization
- **Remote Models**: `src/remote_file.rs:34-89` - Downloadable model specifications
- **Filename Safety**: `src/utils.rs:23-45` - Path traversal prevention via `to_safe_filename()`

**Why This Architecture**: Ensures data integrity across model downloads while preventing security issues. The `is_default()` YAML optimization reduces configuration file size, and strict format validation maintains consistency across HubService, DataService, and CLI interactions.

## OAuth2 Access Control Architecture

BodhiApp's comprehensive access control system coordinates with authentication services through hierarchical roles and scopes:

```rust
// Role hierarchy pattern (see src/role.rs:15-45 for complete implementation)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    User = 0, PowerUser = 1, Manager = 2, Admin = 3,
}
```

### **Cross-Service Authorization Integration**
Access control types enable sophisticated service coordination through these implementations:

- **Role System**: `src/role.rs:67-234` - Hierarchical authorization with `has_access_to()` comparisons
- **Token Scopes**: `src/token_scope.rs:45-156` - OAuth2 scope parsing with "offline_access" validation
- **User Scopes**: `src/user_scope.rs:34-123` - User-based authorization patterns
- **Resource Scopes**: `src/resource_scope.rs:23-89` - Union type for token/user authorization contexts

**Why This Architecture**: Role ordering enables hierarchical authorization decisions across all service boundaries. Case-sensitive scope parsing ensures OAuth2 standards compliance, while the ResourceScope union type seamlessly handles both token-based and user-based authorization contexts used by AuthService, middleware, and routes.

## OpenAI API Compatibility Framework

BodhiApp's sophisticated parameter system ensures complete OpenAI API compatibility through non-destructive parameter overlay:

```rust
// Non-destructive overlay pattern (see src/oai.rs:156-189 for complete implementation)
impl OAIRequestParams {
    pub fn update(&self, request: &mut CreateChatCompletionRequest) {
        // Only fills missing fields - preserves existing values
    }
}
```

### **Cross-Layer Parameter Coordination**
Parameter objects flow throughout BodhiApp's request processing pipeline via these implementations:

- **Core Parameters**: `src/oai.rs:45-234` - OpenAI parameter validation with range enforcement
- **Builder Pattern**: `src/oai.rs:267-345` - Parameter construction with validation
- **Clap Integration**: `src/oai.rs:378-456` - CLI parameter parsing consistency
- **Serialization**: `src/oai.rs:489-567` - YAML/JSON with `is_default()` optimization

**Why This Architecture**: Ensures parameter precedence (request > alias > defaults) while preserving explicit user parameters. The non-destructive overlay pattern enables flexible model parameter management across web interface, CLI, and API endpoints without losing user-specified values.

## Cross-Crate Integration Architecture

### Service Layer Dependency Patterns
The objs crate enables sophisticated service coordination through shared domain objects with comprehensive error handling:

```rust
// Services coordinate via shared domain objects with comprehensive error integration
pub trait HubService {
    async fn download_model(&self, repo: &Repo, snapshot: &str) -> Result<Vec<HubFile>, HubServiceError>;
    async fn list_models(&self, query: &str) -> Result<Vec<String>, HubServiceError>;
}

pub trait DataService {
    fn save_alias(&self, alias: &Alias) -> Result<PathBuf, DataServiceError>;
    fn load_alias(&self, alias_name: &str) -> Result<Alias, DataServiceError>;
    fn validate_hub_file(&self, hub_file: &HubFile) -> Result<(), DataServiceError>;
}

pub trait AuthService {
    async fn exchange_auth_code(&self, /* OAuth2 parameters */) -> Result<(AccessToken, RefreshToken), AuthServiceError>;
    // Uses Role and Scope types for authorization decisions
}

// Error integration pattern used throughout services
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HubServiceError {
    #[error(transparent)]
    HubApiError(#[from] HubApiError),
    #[error("hub_file_missing")]
    #[error_meta(error_type = ErrorType::NotFound)]
    HubFileNotFound(#[from] HubFileNotFoundError),
}
```

- **Error Propagation**: All service errors implement `AppError` via `errmeta_derive` for consistent HTTP response generation with localized messages
- **Domain Validation**: Services use objs validation rules for request parameters, business logic, and cross-service data consistency
- **Model Coordination**: `HubService` and `DataService` coordinate via shared `Repo`, `HubFile`, and `Alias` types with atomic operations and validation
- **Authentication Flow**: `AuthService` uses `Role` and `Scope` types for complex authorization decisions with hierarchical access control
- **Database Integration**: `DbService` uses objs error types for transaction management, migration support, and consistent error handling
- **Cross-Service Error Flow**: Service errors propagate through objs error system to provide consistent API responses and localized messages

### Route Layer Integration Architecture
Routes depend on objs for comprehensive request/response handling:

- **Parameter Validation**: `OAIRequestParams` enforced across all OpenAI-compatible endpoints
- **Error Response Generation**: `ApiError` provides OpenAI-compatible JSON responses for all service errors
- **Authorization Middleware**: `Role` and `Scope` types enable fine-grained access control across route hierarchies
- **Localized Error Messages**: Multi-language error support delivered to web UI, CLI, and API clients

### Extension Guidelines for Cross-Crate Integration

#### Adding New Domain Objects
When creating new objs types that will be used across crates:

1. **Define with validation**: Include comprehensive validation rules and error types
2. **Implement builders**: Provide builder patterns for complex object construction
3. **Add serialization**: Ensure YAML/JSON serialization with `is_default()` optimization
4. **Create test fixtures**: Add to `test_utils` with appropriate mock data
5. **Update downstream crates**: Coordinate changes across services, routes, and CLI with attention to CLI-specific error translation and builder patterns

#### Extending Error System
For new error types that span multiple crates:

```rust
// 1. Define in appropriate domain module with localization
#[error("custom_validation_error")]
#[error_meta(error_type = ErrorType::BadRequest)]
CustomError { field: String, value: String },

// 2. Add Fluent message templates in all supported languages
// src/resources/en-US/messages.ftl:
// custom_validation_error = Invalid {field}: {value}
```

## Critical Cross-Crate Invariants

### Application-Wide Error Handling Requirements
- **Universal Implementation**: All crates must implement `AppError` for error types to maintain consistent behavior
- **Localization Consistency**: All user-facing errors require Fluent message templates in supported languages
- **Cross-Crate Error Flow**: Errors must propagate cleanly from services through routes to clients
- **Fallback Safety**: `ApiError` provides graceful degradation when localization resources fail

### Model Management Coordination Constraints
- **Canonical Format Enforcement**: `Repo` "user/name" format strictly validated across all model-handling components
- **File System Safety**: Alias filename sanitization prevents path traversal across all components
- **Cache Validation**: `HubFile` validation ensures Hugging Face cache structure integrity
- **Binary Format Safety**: GGUF parsing bounds checking prevents crashes across service and CLI usage

### Authentication System Integration Requirements
- **Role Hierarchy Consistency**: Authorization ordering maintained across all service and route contexts
- **OAuth2 Standards Compliance**: Scope parsing ensures compatibility with external identity providers
- **Cross-Service Security**: `TokenScope` and `ResourceScope` enable secure authorization across service boundaries
- **Security Enforcement**: Case-sensitive parsing and "offline_access" requirements maintain security standards

## Commands

**Testing**: `cargo test -p objs` (requires Python 3 for data generators)
**Testing with test-utils**: `cargo test -p objs --features test-utils` (enables comprehensive cross-crate testing infrastructure)
**Building**: Standard `cargo build -p objs`
**Building with test-utils**: `cargo build -p objs --features test-utils` (includes testing utilities for downstream crates)
