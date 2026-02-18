# PACKAGE.md - objs

This document serves as a rich index and navigation aid for the `objs` crate, providing file references and implementation patterns for BodhiApp's **universal foundation layer**.

## BodhiApp Domain Architecture Position

The `objs` crate serves as the **architectural keystone** for BodhiApp, providing foundational types used throughout the entire application ecosystem. Every other crate depends on `objs` for:

- **Cross-Crate Consistency**: Unified domain entities across services, routes, CLI, and desktop components
- **Error System Integration**: Centralized error handling with user-friendly thiserror templates
- **External API Bridging**: Standardized interfaces for OpenAI, Hugging Face, and OAuth2 systems
- **Business Rule Enforcement**: Domain validation and type safety across all application layers

## Centralized Error System Implementation

BodhiApp's error architecture uses domain-specific error enums rather than generic HTTP status code wrappers. Each domain area defines its own error enum with appropriate `ErrorType` variants, ensuring errors carry contextual meaning alongside proper HTTP status codes.

```rust
// Domain-specific error enum pattern (see src/error/objs.rs)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum EntityError {
  #[error("{0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}
```

### **Consolidated IoError Enum**
Previously 6 separate IO error structs (IoError struct, IoWithPathError, IoDirCreateError, IoFileReadError, IoFileWriteError, IoFileDeleteError), now consolidated into a single `IoError` enum with 6 variants and convenience constructors:

```rust
// Unified IO error enum (see src/error/objs.rs)
pub enum IoError {
  Io { source: std::io::Error },
  WithPath { source: std::io::Error, path: String },
  DirCreate { source: std::io::Error, path: String },
  FileRead { source: std::io::Error, path: String },
  FileWrite { source: std::io::Error, path: String },
  FileDelete { source: std::io::Error, path: String },
}
// Convenience: IoError::file_read(source, path)
```

### **Cross-Crate Error Flow Architecture**
The error system coordinates across all BodhiApp layers through these key files:

- **Error Definition**: `src/error/objs.rs` - Domain error types: `EntityError`, `ObjValidationError`, `IoError`, `BuilderError`, `SerdeJsonError`, `SerdeYamlError`, `ReqwestError`, `JsonRejectionError`, `RwLockReadError`, `AppRegInfoMissingError`
- **API Conversion**: `src/error/error_api.rs` - `ApiError` envelope converting any `AppError` to OpenAI-compatible JSON responses. Tests use a test-only `TestError` enum to verify the conversion pipeline.
- **Error Trait & Types**: `src/error/common.rs` - `ErrorType` enum (HTTP status mapping), `AppError` trait, `ErrorMessage` struct
- **OpenAI Format**: `src/error/error_oai.rs` - `OpenAIApiError` and `ErrorBody` for OpenAI-compatible error responses with utoipa schema annotations
**Important**: Generic HTTP wrapper structs (`BadRequestError`, `NotFoundError`, `InternalServerError`, `UnauthorizedError`, `ConflictError`, `UnprocessableEntityError`, `ServiceUnavailableError`) have been removed. Use domain-specific error enums with `#[error_meta(error_type = ErrorType::...)]` instead.

**Error Message Guidelines**:
- Write user-friendly messages in sentence case ending with a period
- Use `{field}` syntax for named field interpolation
- Use `{0}` for positional field interpolation
- Messages should be clear and actionable

**Why This Architecture**: Domain-specific enums carry contextual meaning that generic HTTP wrappers cannot. The consolidated `IoError` enum simplifies pattern matching and reduces boilerplate while retaining operation-specific context through its variants. Error propagation flows seamlessly from services through routes to clients with consistent messages and OpenAI API compatibility.

## GGUF Binary Format System

BodhiApp's comprehensive GGUF support enables safe local AI model management through specialized parsing:

```rust
// Core pattern (see src/gguf/metadata.rs for complete implementation)
let metadata = GGUFMetadata::new(&path)?;
let chat_template = metadata.get_value("tokenizer.chat_template")?.as_str()?;
```

### **Service Integration Architecture**
GGUF parsing integrates throughout BodhiApp's model management pipeline via these implementations:

- **Core Parser**: `src/gguf/metadata.rs` - Memory-mapped file access with bounds checking
- **Value System**: `src/gguf/value.rs` - Typed accessors with endian autodetection
- **Error Handling**: `src/gguf/error.rs` - Localized diagnostics for file corruption
- **Constants**: `src/gguf/constants.rs` - Magic numbers and version validation

**Why This Architecture**: Prevents crashes on corrupted model files while providing detailed diagnostics. Cross-platform endian support enables model sharing between different architectures used by HubService, DataService, and CLI components.

## Model Ecosystem Management Architecture

BodhiApp's sophisticated model management system coordinates between external repositories and local storage through domain objects:

```rust
// Core patterns (see respective files for complete implementations)
impl TryFrom<PathBuf> for HubFile {  // src/hub_file.rs
  fn try_from(path: PathBuf) -> Result<Self, HFCachePathError> {
    // Validates HF cache structure: .../models--<user>--<name>/snapshots/<hash>/<file>
  }
}
```

### **Cross-Service Model Coordination**
Model entities enable complex service interactions through these key implementations:

- **Repository Format**: `src/repo.rs` - Enforces "user/name" format with validation
- **HubFile Management**: `src/hub_file.rs` - Hugging Face cache structure validation
- **Alias System**: `src/alias.rs` - YAML configuration with filename sanitization
- **Remote Models**: `src/remote_file.rs` - Downloadable model specifications
- **Filename Safety**: `src/utils.rs` - Path traversal prevention via `to_safe_filename()`

**Why This Architecture**: Ensures data integrity across model downloads while preventing security issues. The `is_default()` YAML optimization reduces configuration file size, and strict format validation maintains consistency across HubService, DataService, and CLI interactions.

## OAuth2 Access Control Architecture

BodhiApp's comprehensive access control system coordinates with authentication services through hierarchical roles and scopes:

```rust
// Role hierarchy pattern (see src/role.rs for complete implementation)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
  User = 0, PowerUser = 1, Manager = 2, Admin = 3,
}
```

### **Cross-Service Authorization Integration**
Access control types enable sophisticated service coordination through these implementations:

- **Role System**: `src/role.rs` - Hierarchical authorization with `has_access_to()` comparisons
- **Token Scopes**: `src/token_scope.rs` - OAuth2 scope parsing with "offline_access" validation
- **User Scopes**: `src/user_scope.rs` - User-based authorization patterns
- **Resource Scopes**: `src/resource_scope.rs` - Union type for token/user authorization contexts

**Why This Architecture**: Role ordering enables hierarchical authorization decisions across all service boundaries. Case-sensitive scope parsing ensures OAuth2 standards compliance, while the ResourceScope union type seamlessly handles both token-based and user-based authorization contexts used by AuthService, middleware, and routes.

## OpenAI API Compatibility Framework

BodhiApp's sophisticated parameter system ensures complete OpenAI API compatibility through non-destructive parameter overlay:

```rust
// Non-destructive overlay pattern (see src/oai.rs for complete implementation)
impl OAIRequestParams {
  pub fn update(&self, request: &mut CreateChatCompletionRequest) {
    // Only fills missing fields - preserves existing values
  }
}
```

### **Cross-Layer Parameter Coordination**
Parameter objects flow throughout BodhiApp's request processing pipeline via these implementations:

- **Core Parameters**: `src/oai.rs` - OpenAI parameter validation with range enforcement
- **Builder Pattern**: `src/oai.rs` - Parameter construction with validation
- **Clap Integration**: `src/oai.rs` - CLI parameter parsing consistency
- **Serialization**: `src/oai.rs` - YAML/JSON with `is_default()` optimization

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
  #[error("Hub file not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  HubFileNotFound(#[from] HubFileNotFoundError),
}
```

- **Error Propagation**: All service errors implement `AppError` via `errmeta_derive` for consistent HTTP response generation
- **Domain Validation**: Services use objs validation rules for request parameters, business logic, and cross-service data consistency
- **Model Coordination**: `HubService` and `DataService` coordinate via shared `Repo`, `HubFile`, and `Alias` types with atomic operations and validation
- **Authentication Flow**: `AuthService` uses `Role` and `Scope` types for complex authorization decisions with hierarchical access control
- **Database Integration**: `DbService` uses objs error types for transaction management, migration support, and consistent error handling
- **Cross-Service Error Flow**: Service errors propagate through objs error system to provide consistent API responses

### Route Layer Integration Architecture
Routes depend on objs for comprehensive request/response handling:

- **Parameter Validation**: `OAIRequestParams` enforced across all OpenAI-compatible endpoints
- **Error Response Generation**: `ApiError` provides OpenAI-compatible JSON responses for all service errors
- **Authorization Middleware**: `Role` and `Scope` types enable fine-grained access control across route hierarchies
- **User-Friendly Error Messages**: Error messages via thiserror templates delivered to web UI, CLI, and API clients

### Extension Guidelines for Cross-Crate Integration

#### Adding New Domain Objects
When creating new objs types that will be used across crates:

1. **Define with validation**: Include comprehensive validation rules and error types
2. **Implement builders**: Provide builder patterns for complex object construction
3. **Add serialization**: Ensure YAML/JSON serialization with `is_default()` optimization
4. **Create test fixtures**: Add to `test_utils` with appropriate mock data
5. **Update downstream crates**: Coordinate changes across services, routes, and CLI with attention to CLI-specific error translation and builder patterns

#### Extending Error System
For new error types, define domain-specific error enums rather than generic HTTP wrapper structs:

```rust
// Define domain-specific enum in appropriate module (see src/error/objs.rs)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MyDomainError {
  #[error("Invalid {field}: {value}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidField { field: String, value: String },

  #[error("{0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}
```

**Error Message Guidelines**:
- Use sentence case and end with a period
- Use `{field}` for named field interpolation
- Messages should be clear and actionable
- Never create generic HTTP wrapper structs (e.g., `BadRequestError`) -- use domain enum variants with `ErrorType` annotations instead

## Critical Cross-Crate Invariants

### Application-Wide Error Handling Requirements
- **Universal Implementation**: All crates must implement `AppError` for error types to maintain consistent behavior
- **Domain-Specific Enums**: Use domain-specific error enums with `ErrorType` annotations, not generic HTTP wrapper structs. Generic wrappers (`BadRequestError`, `NotFoundError`, etc.) have been removed.
- **Unified IO Errors**: Use `IoError` enum variants with convenience constructors for all IO operations
- **Message Guidelines**: Error messages should be user-friendly, sentence case, ending with a period
- **Field Interpolation**: Use `{field}` for named fields and `{0}` for positional fields
- **Cross-Crate Error Flow**: Errors must propagate cleanly from services through routes to clients

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

## Test Utilities Implementation Index

### Core Test Infrastructure

The `test_utils` module provides comprehensive testing support through rstest-based fixtures and domain-specific builders. All test utilities are conditionally compiled with the `test-utils` feature flag.

#### **Module Organization**: `src/test_utils/mod.rs`
```rust
// Core module exports with conditional compilation
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;
```

**Why This Architecture**: Enables test utilities in both feature-enabled builds and regular test runs while excluding them from production builds.

### Domain Object Test Builders

#### **Model Management Testing**: `src/test_utils/objs.rs`

**Repository and Model Builders**:
```rust
impl Repo {
  pub const LLAMA3: &str = "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF";
  pub const TESTALIAS: &str = "MyFactory/testalias-gguf";
  
  pub fn testalias() -> Repo {
    Repo::from_str(Self::TESTALIAS).unwrap()
  }
}
```

**HubFile Test Fixtures**:
```rust
impl HubFileBuilder {
  pub fn testalias() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
  }
}
```

**Alias Configuration Builders**:
```rust
impl AliasBuilder {
  pub fn testalias() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct")
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(SNAPSHOT)
  }
}
```

**Why This Architecture**: Provides deterministic test data with consistent snapshot identifiers and realistic model configurations. The builder pattern enables flexible test object construction with sensible defaults.

### Environment and Fixture Management

#### **Temporary Directory Fixtures**: `src/test_utils/bodhi.rs`
```rust
#[fixture]
pub fn temp_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

#[fixture]
pub fn empty_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  std::fs::create_dir_all(&dst_path).unwrap();
  temp_dir
}
```

**Why This Architecture**: Provides isolated test environments with realistic directory structures. The `copy_test_dir` function creates complete Bodhi home environments for comprehensive testing.

#### **Hugging Face Cache Simulation**: `src/test_utils/hf.rs`
```rust
#[fixture]
pub fn temp_hf_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_dir
}

#[fixture]
pub fn empty_hf_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("huggingface").join("hub");
  std::fs::create_dir_all(&dst_path).unwrap();
  temp_dir
}
```

**Why This Architecture**: Simulates realistic Hugging Face cache structures enabling comprehensive model management testing. The fixtures provide both populated and empty cache scenarios.

### File System and IO Testing

#### **Test Data Management**: `src/test_utils/io.rs`
```rust
static COPY_OPTIONS: CopyOptions = CopyOptions {
  overwrite: true,
  skip_exist: false,
  copy_inside: true,
  content_only: false,
  buffer_size: 64000,
  depth: 0,
};

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  copy(src_path, dst_path, &COPY_OPTIONS).unwrap();
}
```

**Why This Architecture**: Provides reliable test data copying with consistent options. The function uses CARGO_MANIFEST_DIR for portable path resolution across different build environments.

#### **HTTP Response Parsing**: `src/test_utils/http.rs`
```rust
pub async fn parse<T: serde::de::DeserializeOwned>(response: Response<Body>) -> T {
  let bytes = response.into_body().collect().await.unwrap().to_bytes();
  let str = String::from_utf8_lossy(&bytes);
  serde_json::from_str(&str).unwrap()
}

pub async fn parse_txt(response: Response<Body>) -> String {
  let bytes = response.into_body().collect().await.unwrap().to_bytes();
  String::from_utf8_lossy(&bytes).to_string()
}
```

**Why This Architecture**: Enables type-safe response parsing for integration tests. The functions handle both JSON deserialization and plain text extraction from HTTP responses.

### Error Message Testing

Error messages are validated directly using `error.to_string()`:

```rust
#[test]
fn test_error_message() {
  let error = MyError::ValidationFailed { field: "name".to_string() };
  assert_eq!("Validation failed for field: name.", error.to_string());
}
```

**Why This Architecture**: Error messages are defined inline in thiserror `#[error("...")]` attributes, enabling straightforward testing without localization service dependencies.

### Data Generation and Python Integration

#### **Python Script Execution**: `src/test_utils/test_data.rs`

**Script Execution Infrastructure**:
```rust
pub fn exec_py_script(cwd: &str, script: &str) {
  let output = Command::new("python")
    .arg(script)
    .current_dir(cwd)
    .output()
    .expect("Failed to execute Python script");

  if !output.status.success() {
    panic!("Python script {}/{} failed with status: {}, stderr: {}", 
           cwd, script, output.status, String::from_utf8_lossy(&output.stderr));
  }
}
```

**Test Data Generation Fixtures**:
```rust
#[fixture]
#[once]
pub fn generate_test_data_gguf_metadata() -> () {
  exec_py_script(env!("CARGO_MANIFEST_DIR"), "tests/scripts/test_data_gguf_metadata.py");
}
```

**Why This Architecture**: Integrates Python data generation with Rust testing through controlled script execution. The `#[once]` annotation ensures expensive generation operations run only once per test session.

### Logging and Tracing Support

#### **Test Logging Configuration**: `src/test_utils/logs.rs`
```rust
#[fixture]
#[once]
pub fn enable_tracing() -> () {
  tracing_subscriber::fmt()
    .with_test_writer()
    .with_span_events(FmtSpan::FULL)
    .with_env_filter("tower_sessions=off,tower_sessions_core=off,objs=trace")
    .init();
}
```

**Why This Architecture**: Provides consistent test logging configuration with appropriate filtering. The fixture enables comprehensive tracing for objs while suppressing noisy external crates.

## Test Data Organization

### **Python Generation Scripts**: `tests/scripts/`
- **GGUF Metadata Generation**: `test_data_gguf_metadata.py` - Creates binary GGUF files with controlled metadata
- **Chat Template Generation**: `test_data_chat_template.py` - Generates tokenizer configuration test data  
- **GGUF File Generation**: `test_data_gguf_files.py` - Creates endian-specific binary test files
- **Test Reader Utilities**: `test_reader.py` - Provides Python utilities for GGUF data validation

### **Mock Data Structures**: `tests/data/`
- **Bodhi Configuration**: `tests/data/bodhi/` - Complete Bodhi home directory with aliases and models
- **Hugging Face Cache**: `tests/data/huggingface/hub/` - Realistic HF cache structure with snapshots
- **GGUF Test Files**: `tests/data/gguf/` - Binary format test files with endian variants
- **Chat Templates**: `tests/data/gguf-chat-template/` - GGUF files with various chat template scenarios
- **Tokenizer Configs**: `tests/data/tokenizers/` - Real tokenizer configurations from major models

**Why This Organization**: Provides comprehensive test coverage across all model management scenarios with realistic data structures and controlled test conditions.

## Critical Test Utilities Usage Patterns

### **Cross-Crate Testing Integration**
Downstream crates should enable test utilities through Cargo.toml feature specification:
```toml
[dev-dependencies]
objs = { workspace = true, features = ["test-utils"] }
```

### **Fixture Dependency Injection**
Tests should use rstest fixtures for consistent environment setup:
```rust
#[rstest]
fn test_model_loading(temp_hf_home: TempDir) {
  // Test uses isolated HF cache
}
```

### **Error Message Testing**
Error messages are tested directly using `error.to_string()`:
```rust
#[test]
fn test_error_message() {
  let error = MyError::NotFound { id: "123".to_string() };
  assert_eq!("Resource not found: 123.", error.to_string());
}
```

**Why These Patterns**: Ensures consistent test isolation and straightforward error message validation without localization service dependencies.
