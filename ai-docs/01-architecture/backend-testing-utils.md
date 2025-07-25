# Backend Testing Utils Pattern

## Overview

The Bodhi App implements a unique and powerful testing pattern using Rust feature flags to create reusable test utilities that can be shared across multiple crates. This pattern enables downstream crates to access mock objects, builders, stubs, and test fixtures from upstream crates without code duplication, while maintaining clean separation between production and test code.

## Architecture Pattern

### Core Concept

The `test-utils` feature flag pattern allows us to:

1. **Conditional Compilation**: Test utilities are only compiled when needed
2. **Cross-Crate Sharing**: Downstream crates can access test utilities from dependencies
3. **Zero Production Overhead**: Test code is completely excluded from production builds
4. **Centralized Test Objects**: Common test objects are defined once and reused everywhere

### Feature Flag Configuration

The pattern uses two complementary conditional compilation directives:

<augment_code_snippet path="crates/objs/src/lib.rs" mode="EXCERPT">
````rust
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;
````
</augment_code_snippet>

This dual configuration ensures:
- **With `test-utils` feature**: Test utilities are always available (for downstream crates)
- **Without `test-utils` feature**: Test utilities are only available during `cargo test` (for internal tests)

## Implementation Details

### Base Crate Setup (objs)

The `objs` crate serves as the foundation, containing all domain objects and their test utilities.

#### Cargo.toml Configuration

<augment_code_snippet path="crates/objs/Cargo.toml" mode="EXCERPT">
````toml
[features]
test-utils = [
  "dircpy",
  "dirs", 
  "fs_extra",
  "http-body-util",
  "rstest",
  "tempfile",
  "tracing-subscriber",
]
````
</augment_code_snippet>

The `test-utils` feature enables optional dependencies that are only needed for testing.

#### Test Utilities Module Structure

<augment_code_snippet path="crates/objs/src/test_utils/mod.rs" mode="EXCERPT">
````rust
mod bodhi;
mod envs;
mod error;
mod hf;
mod http;
mod io;
mod l10n;
mod logs;
mod objs;
mod test_data;

pub use bodhi::*;
pub use envs::*;
pub use error::*;
pub use hf::*;
pub use http::*;
pub use io::*;
pub use l10n::*;
pub use logs::*;
pub use objs::*;
pub use test_data::*;
````
</augment_code_snippet>

### Test Object Builders and Fixtures

The pattern provides rich builder implementations and test fixtures:

#### Domain Object Builders

<augment_code_snippet path="crates/objs/src/test_utils/objs.rs" mode="EXCERPT">
````rust
impl AliasBuilder {
  pub fn testalias() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct")
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(SNAPSHOT)
      .source(AliasSource::User)
      .request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}

impl Alias {
  pub fn testalias() -> Alias {
    AliasBuilder::testalias().build().unwrap()
  }
}
````
</augment_code_snippet>

#### Test Fixtures with rstest

<augment_code_snippet path="crates/objs/src/test_utils/bodhi.rs" mode="EXCERPT">
````rust
#[fixture]
pub fn temp_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

#[fixture]
pub fn temp_dir() -> TempDir {
  build_temp_dir()
}
````
</augment_code_snippet>

### Downstream Crate Integration (services)

Downstream crates can leverage the test utilities by enabling the feature flag.

#### Dependency Configuration

<augment_code_snippet path="crates/services/Cargo.toml" mode="EXCERPT">
````toml
[dev-dependencies]
objs = { workspace = true, features = ["test-utils"] }

[features]
test-utils = [
  "rstest",
  "mockall", 
  "once_cell",
  "rsa",
  "tap",
  "tempfile",
  "anyhow",
  "objs/test-utils",  # Enable upstream test-utils
  "tokio",
]
````
</augment_code_snippet>

#### Using Upstream Test Utilities

<augment_code_snippet path="crates/services/src/test_utils/data.rs" mode="EXCERPT">
````rust
use objs::{test_utils::temp_bodhi_home, Alias, RemoteModel};

#[fixture]
pub fn test_data_service(
  temp_bodhi_home: TempDir,
  test_hf_service: TestHfService,
) -> TestDataService {
  let inner = LocalDataService::new(
    temp_bodhi_home.path().join("bodhi"),
    Arc::new(test_hf_service),
  );
  TestDataService {
    temp_bodhi_home,
    inner,
  }
}
````
</augment_code_snippet>

## Benefits and Advantages

### 1. **Zero Code Duplication**
- Test objects are defined once in the base crate
- All downstream crates reuse the same test data
- Consistent test scenarios across the entire codebase

### 2. **Type Safety**
- Test utilities use the same types as production code
- Compile-time verification of test object validity
- Refactoring automatically updates all test usages

### 3. **Maintainability**
- Single source of truth for test data
- Changes to domain objects automatically propagate to tests
- Easy to add new test scenarios globally

### 4. **Performance**
- Test utilities are completely excluded from production builds
- No runtime overhead in release builds
- Conditional compilation ensures optimal binary size

### 5. **Flexibility**
- Each crate can extend test utilities with its own patterns
- Composable test fixtures using rstest
- Easy to create complex test scenarios

## Advanced Usage Patterns

### 1. **Simple Object Creation**

```rust
use objs::test_utils::*;

#[test]
fn test_alias_functionality() {
    let alias = Alias::testalias();
    assert_eq!(alias.alias(), "testalias:instruct");
}
```

### 2. **Builder Pattern for Customization**

```rust
use objs::test_utils::*;

#[test]
fn test_custom_alias() {
    let alias = AliasBuilder::testalias()
        .alias("custom:model")
        .build()
        .unwrap();
    assert_eq!(alias.alias(), "custom:model");
}
```

### 3. **Fixture-Based Testing**

```rust
use objs::test_utils::*;
use rstest::rstest;

#[rstest]
fn test_with_temp_directory(temp_bodhi_home: TempDir) {
    let bodhi_path = temp_bodhi_home.path().join("bodhi");
    assert!(bodhi_path.exists());
}
```

### 5. **Setup L10n Fixture Usage**

When using the `setup_l10n` fixture in rstest tests, there are two patterns depending on whether you need to interact with the localization service:

**Pattern 1: When you need to use the setup_l10n object**
```rust
use objs::{test_utils::setup_l10n, FluentLocalizationService};
use rstest::rstest;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_with_localization(
  #[from(setup_l10n)] setup_l10n: &Arc<FluentLocalizationService>,
) -> anyhow::Result<()> {
  // Use setup_l10n object in your test
  let state = create_test_state(setup_l10n, &config).await?;
  // ... rest of test
  Ok(())
}
```

**Pattern 2: When you don't interact with setup_l10n directly**
```rust
use objs::{test_utils::setup_l10n, FluentLocalizationService};
use rstest::rstest;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_without_localization_interaction(
  #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
) -> anyhow::Result<()> {
  // setup_l10n is available but not directly used
  // The underscore prefix prevents unused variable warnings
  let result = some_test_operation();
  assert!(result.is_ok());
  Ok(())
}
```
### 4. **Cross-Crate Test Integration**

```rust
use objs::test_utils::*;
use services::test_utils::*;

#[rstest]
fn test_service_integration(
    temp_bodhi_home: TempDir,
    test_data_service: TestDataService,
) {
    let alias = Alias::testalias();
    let result = test_data_service.save_alias(alias);
    assert!(result.is_ok());
}
```

### 5. **Complex Service Composition**

<augment_code_snippet path="crates/services/src/test_utils/app.rs" mode="EXCERPT">
````rust
#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  pub temp_home: Option<Arc<TempDir>>,
  pub setting_service: Option<Arc<dyn SettingService>>,
  pub hub_service: Option<Arc<dyn HubService>>,
  pub data_service: Option<Arc<dyn DataService>>,
  pub auth_service: Option<Arc<dyn AuthService>>,
  // ... other services
}
````
</augment_code_snippet>

This pattern enables complex service composition for integration testing.

### 6. **Mock and Stub Implementations**

<augment_code_snippet path="crates/services/src/test_utils/hf.rs" mode="EXCERPT">
````rust
#[derive(Debug)]
pub struct TestHfService {
  _temp_dir: TempDir,
  inner: HfHubService,
  inner_mock: MockHubService,
  allow_downloads: bool,
}

impl HubService for TestHfService {
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile> {
    if self.allow_downloads {
      self.inner.download(repo, filename, snapshot)
    } else {
      self.inner_mock.download(repo, filename, snapshot)
    }
  }
}
````
</augment_code_snippet>

### 7. **Authentication Test Utilities**

<augment_code_snippet path="crates/services/src/test_utils/auth.rs" mode="EXCERPT">
````rust
impl AppRegInfoBuilder {
  pub fn test_default() -> Self {
    Self::default()
      .public_key(PUBLIC_KEY_BASE64.to_string())
      .issuer(ISSUER.to_string())
      .client_id(TEST_CLIENT_ID.to_string())
      .client_secret(TEST_CLIENT_SECRET.to_string())
      .alg(Algorithm::RS256)
      .kid(TEST_KID.to_string())
      .to_owned()
  }
}
````
</augment_code_snippet>

## Advanced Patterns and Techniques

### 1. **Layered Service Architecture Testing**

The pattern enables sophisticated testing of layered service architectures:

<augment_code_snippet path="crates/services/src/test_utils/app.rs" mode="EXCERPT">
````rust
impl AppServiceStubBuilder {
  pub fn with_data_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let hub_service = self.with_hub_service().hub_service.clone().unwrap().unwrap().clone();
    let bodhi_home = temp_home.path().join("bodhi");
    copy_test_dir("tests/data/bodhi", &bodhi_home);
    let data_service = LocalDataService::new(bodhi_home, hub_service);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }
}
````
</augment_code_snippet>

### 2. **Time-Controlled Testing**

<augment_code_snippet path="crates/services/src/test_utils/db.rs" mode="EXCERPT">
````rust
#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0  // Always returns the same time
  }
}
````
</augment_code_snippet>

This enables deterministic testing of time-dependent functionality.

### 3. **Event-Driven Testing**

<augment_code_snippet path="crates/services/src/test_utils/db.rs" mode="EXCERPT">
````rust
pub struct TestDbService {
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
}

impl TestDbService {
  pub fn subscribe(&self) -> Receiver<String> {
    self.event_sender.subscribe()
  }

  fn notify(&self, event: &str) {
    let _ = self.event_sender.send(event.to_string());
  }
}
````
</augment_code_snippet>

### 4. **Integration Test Fixtures**

<augment_code_snippet path="crates/integration-tests/tests/utils/live_server_utils.rs" mode="EXCERPT">
````rust
#[fixture]
pub async fn llama2_7b_setup(
  #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) -> anyhow::Result<(TempDir, Arc<dyn AppService>)> {
  // Complex setup combining multiple services
  let temp_dir = tempfile::tempdir().unwrap();
  let bodhi_home = cache_dir.join("bodhi");
  copy_test_dir("tests/data/live/bodhi", &bodhi_home);

  // Build complete application service
  let service = DefaultAppService::new(
    Arc::new(setting_service),
    hub_service,
    Arc::new(data_service),
    // ... other services
  );
  Ok((temp_dir, Arc::new(service)))
}
````
</augment_code_snippet>

## Best Practices

### 1. **Feature Flag Naming**
- Use consistent `test-utils` naming across all crates
- Include upstream feature flags in downstream dependencies
- Document feature flag purpose in Cargo.toml

### 2. **Module Organization**
- Group related test utilities in focused modules
- Use clear, descriptive names for test objects
- Provide both builder and direct construction methods

### 3. **Test Data Management**
- Use constants for commonly used test values
- Provide realistic but deterministic test data
- Include edge cases and error scenarios

### 4. **Service Composition**
- Create composable service stubs that can be mixed and matched
- Use builder patterns for complex service setup
- Provide sensible defaults while allowing customization

### 5. **Mock Strategy**
- Combine real implementations with mocks for hybrid testing
- Use feature flags to control mock vs real behavior
- Provide clear interfaces for expectation setting

### 6. **Documentation**
- Document test utility purpose and usage
- Provide examples for complex test scenarios
- Explain relationships between test objects

## Real-World Examples

### 1. **API Route Testing**

<augment_code_snippet path="crates/routes_app/src/routes_models.rs" mode="EXCERPT">
````rust
#[rstest]
#[awt]
#[tokio::test]
async fn test_list_local_aliases_handler(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models").body(Body::empty()).unwrap())
    .await?
    .json::<Value>()
    .await?;
  assert_eq!(1, response["page"]);
  assert_eq!(30, response["page_size"]);
  Ok(())
}
````
</augment_code_snippet>

### 2. **Integration Test with Authentication**

<augment_code_snippet path="crates/integration-tests/tests/utils/live_server_utils.rs" mode="EXCERPT">
````rust
pub async fn create_authenticated_session(
  app_service: &Arc<dyn AppService>,
  access_token: &str,
  refresh_token: &str,
) -> anyhow::Result<String> {
  let session_service = app_service.session_service();
  let session_data = maplit::hashmap! {
    SESSION_KEY_ACCESS_TOKEN.to_string() => Value::String(access_token.to_string()),
    SESSION_KEY_REFRESH_TOKEN.to_string() => Value::String(refresh_token.to_string()),
  };
  // Create session and return ID
}
````
</augment_code_snippet>

### 3. **Cross-Layer Response Testing**

<augment_code_snippet path="crates/routes_app/src/test_utils/alias_response.rs" mode="EXCERPT">
````rust
impl AliasResponse {
  pub fn llama3() -> Self {
    AliasResponseBuilder::default()
      .alias("llama3:instruct")
      .repo(Repo::LLAMA3)
      .filename(Repo::LLAMA3_Q8)
      .request_params(
        OAIRequestParamsBuilder::default()
          .stop(vec!["<|start_header_id|>".to_string()])
          .build()
          .unwrap(),
      )
      .build()
      .unwrap()
  }
}
````
</augment_code_snippet>

## Implementation Checklist

When implementing this pattern in a new crate:

- [ ] Add `test-utils` feature to Cargo.toml
- [ ] Include necessary optional dependencies
- [ ] Create `test_utils` module with conditional compilation
- [ ] Implement builder patterns for domain objects
- [ ] Add rstest fixtures for common scenarios
- [ ] Export all utilities through module re-exports
- [ ] Update downstream crates to use new utilities
- [ ] Document usage patterns and examples
- [ ] Create service stubs and mocks for complex dependencies
- [ ] Implement time-controlled and event-driven test utilities
- [ ] Provide integration test fixtures for end-to-end scenarios

## Key Insights and Design Principles

### 1. **Conditional Compilation Strategy**
The dual `#[cfg]` approach is sophisticated:
- `#[cfg(feature = "test-utils")]` - Always available when feature is enabled
- `#[cfg(all(not(feature = "test-utils"), test))]` - Available during internal testing only

This ensures test utilities are accessible to downstream crates when needed, but excluded from production builds.

### 2. **Service Composition Architecture**
The pattern enables complex service composition testing:
- **Layered Dependencies**: Services can depend on other services through test utilities
- **Hybrid Testing**: Mix real implementations with mocks for targeted testing
- **Builder Patterns**: Fluent APIs for complex test scenario setup

### 3. **Cross-Crate Type Safety**
- Test objects use the same types as production code
- Refactoring automatically updates all test usages
- Compile-time verification of test object validity

### 4. **Scalability Patterns**
- **Modular Organization**: Each crate extends the pattern with domain-specific utilities
- **Fixture Composition**: rstest fixtures can be composed for complex scenarios
- **Event-Driven Testing**: Support for testing asynchronous and event-driven systems

## Conclusion

The `test-utils` feature flag pattern represents a sophisticated approach to testing in multi-crate Rust projects. It goes beyond simple object creation to enable:

- **Complex Service Architecture Testing**: Full application service composition with configurable dependencies
- **Time-Controlled Testing**: Deterministic testing of time-dependent functionality
- **Event-Driven Testing**: Support for testing asynchronous systems with event notifications
- **Integration Testing**: End-to-end testing with real database connections and HTTP servers
- **Authentication Testing**: Complete OAuth2 flow testing with JWT token generation

The pattern eliminates code duplication while maintaining type safety and zero production overhead. It scales from simple unit tests to complex integration scenarios, making it particularly valuable for large Rust applications with multiple architectural layers.

This approach demonstrates how Rust's conditional compilation features can be leveraged to create powerful, maintainable testing infrastructure that grows with the codebase complexity while preserving the performance characteristics of production builds.
