# Mock Service Patterns with mockall

## Overview

The services crate uses `mockall` for generating mock implementations of service traits. Two mock patterns exist:

1. **`#[automock]` on traits** -- automatic mock generation for simple service traits
2. **`mockall::mock!` macro** -- manual mock for composite traits like `DbService`

## MockDbService (Composite Mock)

`DbService` is a composite of multiple repository traits (`DbCore`, `ModelRepository`, `AccessRepository`, `TokenRepository`, `ToolsetRepository`). The mock is defined manually in `test_utils/db.rs`:

```rust
use crate::test_utils::MockDbService;

let mut mock_db = MockDbService::new();
mock_db
  .expect_list_toolsets()
  .with(eq("user123"))
  .returning(|_| Ok(vec![/* ... */]));
```

### Common MockDbService Expectations

```rust
use mockall::predicate::eq;

// Expect a specific method call with argument matching
mock_db
  .expect_get_api_model_alias()
  .with(eq("my-alias"))
  .returning(|_| Ok(Some(ApiAlias { /* ... */ })));

// Expect method called once and return error
mock_db
  .expect_create_toolset()
  .times(1)
  .returning(|_| Err(DbError::SqlxError(SqlxError(sqlx::Error::RowNotFound))));

// Never expect a method to be called
mock_db
  .expect_delete_toolset()
  .never();
```

## Automock Traits

Most service traits use `#[cfg_attr(test, mockall::automock)]` for automatic mock generation:

```rust
// In the service definition:
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExaService: Send + Sync + std::fmt::Debug {
  async fn search(&self, api_key: &str, request: ExaSearchRequest) -> Result<ExaSearchResponse, ExaError>;
}

// In tests:
use crate::exa_service::MockExaService;

let mut mock_exa = MockExaService::new();
mock_exa
  .expect_search()
  .returning(|_, _| Ok(ExaSearchResponse { /* ... */ }));
```

## ToolService Test Example (Full Pattern)

Shows mocking multiple dependencies for a service:

```rust
use crate::db::MockTimeService;
use crate::test_utils::MockDbService;
use crate::exa_service::MockExaService;
use crate::tool_service::{DefaultToolService, ToolService};
use mockall::predicate::eq;
use std::sync::Arc;

// Sync test -- no #[anyhow_trace]
#[rstest]
fn test_list_all_tool_definitions() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let defs = service.list_all_tool_definitions();

  assert_eq!(1, defs.len());
  assert_eq!("builtin-exa-web-search", defs[0].function.name);
}

// Async test with expectations
#[rstest]
#[tokio::test]
async fn test_list_tools_for_user() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| Ok(vec![test_toolset_row("id1", "user123", "my-exa")]));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let tools = service.list_tools_for_user("user123").await?;

  assert_eq!(1, tools.len());
  Ok(())
}
```

## Predicate Patterns

```rust
use mockall::predicate::{eq, always, function};

// Exact value match
.with(eq("expected_value"))

// Multiple arguments
.with(eq("arg1"), eq("arg2"))

// Any value accepted
.with(always())

// Custom predicate
.with(function(|arg: &str| arg.starts_with("prefix")))
```

## Return Patterns

```rust
// Return a fixed value (closure called each time)
.returning(|_| Ok(vec![]))

// Return exactly once (can use non-Clone values)
.return_once(|_, _, _, _, _| ())

// Return different values on successive calls
.times(1)
.returning(|_| Ok(Some(first_value)));
// Then set up a second expectation for the next call

// Return an error
.returning(|_| Err(MyError::NotFound))
```

## Call Count Verification

```rust
// Expect exactly N calls
.times(1)

// Expect never called
.never()

// Expect at least / at most
.times(1..=3)
```

## MockSettingsChangeListener

For testing setting change notifications:

```rust
use crate::MockSettingsChangeListener;
use mockall::predicate::eq;

let mut mock_listener = MockSettingsChangeListener::default();
mock_listener
  .expect_on_change()
  .with(
    eq("TEST_KEY"),
    eq(Some(Value::String("old".to_string()))),
    eq(SettingSource::SettingsFile),
    eq(Some(Value::String("new".to_string()))),
    eq(SettingSource::SettingsFile),
  )
  .times(1)
  .return_once(|_, _, _, _, _| ());

service.add_listener(Arc::new(mock_listener));
```

## MockAppService

For higher-level tests that need a full `AppService` mock:

```rust
use services::MockAppService;

let mut mock_app = MockAppService::new();
mock_app
  .expect_db_service()
  .returning(|| Arc::new(mock_db));
```

Note: `MockAppService` is auto-generated from the `#[mockall::automock]` on `AppService` trait. For route-layer tests, prefer `AppServiceStubBuilder` instead.
