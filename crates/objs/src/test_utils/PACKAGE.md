# PACKAGE.md - test_utils

This document provides detailed technical information for the `test_utils` module, focusing on BodhiApp-specific testing implementation patterns used across multiple crates and sophisticated cross-crate testing coordination.

## Cross-Crate Error Testing Architecture

### Error Message Validation Pattern
BodhiApp's foundational error testing infrastructure uses thiserror templates with `error.to_string()` validation:

```rust
#[rstest]
fn test_error_message_validation() -> anyhow::Result<()> {
  let error = SomeError::ValidationError {
    field: "username".to_string(),
    reason: "too short".to_string(),
  };

  // Error messages validated via error.to_string()
  let message = error.to_string();
  assert!(message.contains("username"));
  assert!(message.contains("too short"));

  // Error messages should be user-friendly, sentence case, end with period
  assert!(message.ends_with("."));
  Ok(())
}
```

### Downstream Crate Integration Implementation
Error testing patterns enable sophisticated cross-crate testing scenarios:

**Services Crate Usage**:
- Service error testing validates error messages via `error.to_string()` across service boundaries with comprehensive error propagation from `AuthServiceError`, `HubServiceError`, `DataServiceError`, and `DbError`
- `AppServiceStub` tests validate error messages for comprehensive error validation across authentication flows, model management, and database operations
- Database service tests ensure error messages work correctly across transaction boundaries with `TestDbService` and event broadcasting patterns
- Cross-service integration testing validates error flows from `HubService` → `DataService` → `AuthService` with consistent error messaging
- Service composition testing uses objs fixtures for realistic multi-service scenarios including OAuth2 flows, model downloads, and database transactions

**Routes Crate Integration**:
- API endpoint tests validate error responses for OpenAI compatibility using `error.to_string()`
- Middleware tests ensure authentication errors have proper user-friendly messages
- Route integration tests validate error propagation from services through thiserror templates

**CLI Testing Coordination**:
- Command-line interface tests validate help text and error messages
- CLI error handling tests ensure consistent messaging with web interface

**Key Cross-Crate Implementation Features**:
- Error messages defined inline via thiserror `#[error("...")]` templates
- Field interpolation using `{field}` syntax for named fields
- User-friendly messages: sentence case, ending with period
- Test isolation requires careful cleanup coordination across multiple test suites

### Error Testing Patterns for Cross-Crate Testing
Comprehensive error validation supporting application-wide testing:

- **thiserror Template Validation**: Error messages tested via `error.to_string()` throughout codebase
- **Cross-Boundary Validation**: Tests validate error messages work seamlessly across service → route → client boundaries
- **Integration Test Support**: Enables comprehensive end-to-end testing of error flows across entire application stack

## Environment Isolation for Multi-Crate Integration

### Comprehensive Temporary Environment Architecture
BodhiApp's environment fixtures enable sophisticated integration testing across multiple crates:

```rust
#[fixture]
pub fn temp_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

#[fixture]
pub fn temp_hf_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_dir
}

// Example usage in tests:
#[rstest]
fn test_local_model_file_from_pathbuf(temp_hf_home: TempDir) -> anyhow::Result<()> {
  let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
  let filepath = hf_cache
    .join("models--MyFactory--testalias-gguf")
    .join("snapshots")
    .join("5007652f7a641fe7170e0bad4f63839419bd9213")
    .join("testalias.Q8_0.gguf");
  let local_model = HubFile::try_from(filepath)?;
  assert_eq!(expected, local_model);
  Ok(())
}
```

### Cross-Crate Environment Coordination
Environment fixtures support complex downstream testing scenarios:

**Services Crate Integration**:
- `temp_bodhi_home()` provides configuration directory for `DataService` alias testing with realistic YAML configuration files and atomic file operations
- `temp_hf_home()` enables `HubService` cache validation without external dependencies using `OfflineHubService` pattern with local test data
- Environment isolation ensures service tests don't interfere with each other through isolated SQLite databases and temporary directory management
- Service composition testing coordinates environment fixtures with `AppServiceStub` builder pattern for comprehensive integration scenarios
- Database testing uses environment fixtures with `TestDbService` for isolated transaction testing and migration validation
- Session service testing integrates environment fixtures with SQLite session store and secure cookie configuration testing

**Route Testing Support**:
- Configuration environments enable route integration tests with realistic file system state
- Model file fixtures support API endpoint testing for model management routes
- Environment cleanup prevents test data pollution across route test suites

**CLI Integration Testing**:
- Isolated environments enable CLI command testing without affecting system configuration
- Pre-populated test data supports comprehensive CLI functionality validation
- Environment fixtures coordinate with CLI argument parsing and file system operations

**Implementation Features for Cross-Crate Usage**:
- Pre-populated test data copied to temporary directories for realistic testing scenarios
- Automatic cleanup after test completion prevents resource leaks across test runs
- Maintains authentic Hugging Face cache directory structure for model validation testing
- Fixed snapshot hash (`SNAPSHOT = "5007652f7a641fe7170e0bad4f63839419bd9213"`) ensures reproducible tests across all consuming crates

## Cross-Crate Domain Object Factory Architecture

### Universal Repository Constants for Multi-Crate Testing
Comprehensive model constants supporting consistent testing across all BodhiApp crates:

```rust
impl Repo {
  pub const LLAMA3: &str = "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF";
  pub const LLAMA3_Q8: &str = "Meta-Llama-3-8B-Instruct.Q8_0.gguf";
  pub const LLAMA2: &str = "TheBloke/Llama-2-7B-Chat-GGUF";
  pub const TINYLLAMA: &str = "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF";
  pub const TESTALIAS: &str = "MyFactory/testalias-gguf";
  pub const FAKEMODEL: &str = "FakeFactory/fakemodel-gguf";
  
  pub fn llama3() -> Repo {
    Repo::from_str(Self::LLAMA3).unwrap()
  }
  
  pub fn testalias() -> Repo {
    Repo::from_str(Self::TESTALIAS).unwrap()
  }
}
```

### Comprehensive Builder Pattern for Cross-Crate Consistency
Sophisticated builders enable consistent domain object creation across all downstream crates:

```rust
impl HubFileBuilder {
  pub fn testalias() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }
  
  pub fn llama3_tokenizer() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::llama3_tokenizer())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot("c4a54320a52ed5f88b7a2f84496903ea4ff07b45".to_string())
      .size(Some(50977))
      .to_owned()
  }
}

impl AliasBuilder {
  pub fn llama3() -> AliasBuilder {
    let request_params = OAIRequestParamsBuilder::default()
      .stop(vec![
        "<|start_header_id|>".to_string(),
        "<|end_header_id|>".to_string(),
        "<|eot_id|>".to_string(),
      ])
      .build()
      .unwrap();
    let gpt_params = vec!["--n-keep 24".to_string()];
    AliasBuilder::default()
      .alias("llama3:instruct".to_string())
      .repo(Repo::llama3())
      .filename(Repo::LLAMA3_Q8.to_string())
      .snapshot(SNAPSHOT.to_string())
      .source(AliasSource::User)
      .request_params(request_params)
      .context_params(gpt_params)
      .to_owned()
  }
}
```

### Domain Object Usage Across Crates
Domain factories enable sophisticated integration testing:

**Services Crate Coordination**:
- `HubService` tests use repository constants (`Repo::TESTALIAS`, `Repo::LLAMA3`) for consistent model identification across `OfflineHubService` and production implementations
- `DataService` tests leverage alias builders (`AliasBuilder::llama3()`, `AliasBuilder::testalias()`) for YAML serialization validation and atomic file operations
- Service integration tests coordinate via shared domain objects preventing data inconsistencies across `AppServiceStub` composition and mock service expectations
- Authentication service testing uses role and scope builders for comprehensive OAuth2 flow validation and token exchange testing
- Database service testing coordinates domain objects with `TestDbService` for transaction management and migration testing
- Cross-service testing validates domain object flow from `HubService` → `DataService` → `AuthService` with consistent error handling and localization

**Route Integration Testing**:
- API endpoint tests use domain factories for consistent request/response validation
- Route parameter testing validates domain object serialization across HTTP boundaries
- Integration tests ensure API responses match domain object constraints

**CLI Testing Integration**:
- Command-line tests use repository builders for consistent argument parsing validation
- CLI output validation coordinates with domain object string representations
- Integration tests ensure CLI behavior matches API behavior via shared domain objects

**Key Cross-Crate Implementation Patterns**:
- Fixed snapshot hash ensures reproducible model testing across all consuming crates
- Pre-configured builders provide common test scenarios used throughout application testing
- Domain validation coordinated across builders ensures constraint consistency across crate boundaries
- Builder pattern extensibility enables crate-specific customizations while maintaining core consistency

## Cross-Crate Python Integration Architecture

### Advanced Data Generation Pipeline Supporting Multiple Crates
Sophisticated Python integration enabling complex test data generation across BodhiApp:

```rust
pub fn exec_py_script(cwd: &str, script: &str) {
  let output = Command::new("python")
    .arg(script)
    .current_dir(cwd)
    .output()
    .expect("Failed to execute Python script");

  if !output.status.success() {
    panic!(
      "Python script {}/{} failed with status: {}, stderr: {}",
      cwd, script, output.status, String::from_utf8_lossy(&output.stderr)
    );
  }
}
```

### Python Script Integration Across Application Testing
Python data generators support comprehensive multi-crate testing scenarios:

**Cross-Crate GGUF Testing**:
- `generate_test_data_gguf_metadata()`: Creates binary GGUF test files used by services, routes, and CLI testing
- GGUF metadata validation tested across `DataService`, API endpoints, and command-line tools
- Binary format fixtures ensure consistent GGUF parsing behavior across all application layers

**Tokenizer Configuration Testing**:
- `generate_test_data_chat_template()`: Generates tokenizer configurations tested across service and route boundaries
- Chat template validation coordinates between GGUF parsing and OpenAI API compatibility
- Tokenizer fixtures enable comprehensive integration testing of model parameter extraction

**Infrastructure Features for Cross-Crate Usage**:
- Scripts centrally located in `tests/scripts/` directory with shared Python requirements
- Error reporting provides detailed diagnostic output for failures across all consuming crates
- Data generation coordinated across multiple test suites to prevent resource conflicts

## Extension Guidelines for Cross-Crate Integration

### Adding New Cross-Crate Test Fixtures
When creating test utilities that will be used across multiple crates:

1. **Coordinate Cross-Crate Patterns**: Use `#[fixture]` attributes with consistent naming across consuming crates
2. **Maintain Application-Wide Isolation**: Ensure fixtures don't interfere with tests across different crates
3. **Implement Universal Builders**: Provide builder patterns that work consistently across service, route, and CLI testing
4. **Document Cross-Crate Constraints**: Add validation rules and coordination patterns to both CLAUDE.md and PACKAGE.md

### Cross-Crate Python Data Generator Extensions
For new Python generators that will be used by multiple crates:

1. **Centralize Script Location**: Place all scripts in `tests/scripts/` directory for workspace-wide access
2. **Coordinate Dependencies**: Manage Python requirements.txt to prevent conflicts across crate testing
3. **Use Workspace-Level `#[once]` Fixtures**: Expensive data generation shared across multiple crate test suites
4. **Provide Comprehensive Error Context**: Script failures must be diagnosable across all consuming crate contexts

### Application-Wide Cross-Crate Testing Extensions
When extending testing capabilities that span multiple crates:

1. **Error Message Guidelines**: Ensure error messages are user-friendly, sentence case, ending with period
2. **Field Interpolation**: Use `{field}` syntax for named fields and `{0}` for positional fields in error messages
3. **Validate End-to-End Integration**: Test error messages and domain object behavior across complete application stack
4. **Coordinate Feature Dependencies**: Update feature flags across consuming crates when new external dependencies introduced
5. **Maintain Cross-Crate Documentation**: Update both objs documentation and consuming crate documentation when patterns change

## Critical Cross-Crate Testing Patterns

### Application-Wide Error Testing Requirements
Comprehensive error testing patterns used throughout BodhiApp:

```rust
#[rstest]
fn test_error_cross_crate_integration() {
  let error = SomeError::new("field", "value");
  let api_error = ApiError::from(error);
  let message = api_error.message();
  assert!(message.contains("Invalid field"));

  // Validate error propagation across service → route → client boundaries
  let service_result = some_service.process_request(invalid_request).await;
  assert!(service_result.is_err());
  let route_response = handle_service_error(service_result.unwrap_err());
  assert_eq!(route_response.status(), StatusCode::BAD_REQUEST);
}
```

### Cross-Crate Model File Testing Integration
Model validation patterns coordinated across multiple crates:

```rust
#[rstest]
fn test_model_validation_across_services(temp_hf_home: TempDir) {
  let hub_file = HubFileBuilder::testalias()
    .hf_cache(temp_hf_home.path().join("huggingface"))
    .build()
    .unwrap();
  assert!(hub_file.exists());
  
  // Test coordination between HubService and DataService
  let hub_service = HubService::new();
  let data_service = DataService::new();
  
  let downloaded_files = hub_service.download_model(&hub_file.repo, &hub_file.snapshot).await.unwrap();
  let alias = data_service.create_alias_from_hub_file(&downloaded_files[0]).unwrap();
  assert!(data_service.alias_exists(&alias.alias));
}
```

### Cross-Application Environment Testing Coordination
Environment patterns supporting comprehensive integration testing:

```rust
#[rstest]
fn test_configuration_loading_cross_crate(temp_bodhi_home: TempDir) {
  std::env::set_var("BODHI_HOME", temp_bodhi_home.path());
  let config = load_bodhi_config().unwrap();
  assert!(!config.aliases.is_empty());

  // Test configuration coordination across CLI, services, and routes
  let cli_result = cli_command_with_config(&config).await;
  let service_result = service_with_config(&config).await;
  let route_result = route_with_config(&config).await;

  // Ensure consistent behavior across all application layers
  assert_eq!(cli_result.model_count(), service_result.model_count());
  assert_eq!(service_result.available_models(), route_result.available_models());
}
```

### Multi-Crate Integration Testing Requirements
Comprehensive patterns for testing across entire application stack:

- **Service → Route → Client Flow Testing**: Validate complete request/response cycles with user-friendly errors
- **Configuration Consistency Testing**: Ensure CLI, web interface, and API behave consistently with shared configuration
- **Model Management Integration**: Test model download, alias creation, and API access across all application layers
- **Authentication Flow Coordination**: Validate OAuth2 flows work consistently across web and CLI interfaces
- **Error Propagation Validation**: Ensure errors flow correctly from services through routes to user-friendly client messages

## Commands for Cross-Crate Testing

**Full Application Testing**: `cargo test --workspace` (requires Python 3 for data generators)  
**Objs Foundation Testing**: `cargo test -p objs --features test-utils`  
**Cross-Crate Integration**: `cargo test --workspace --features test-utils,integration-tests`