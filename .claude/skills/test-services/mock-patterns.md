# Mock Service Patterns with mockall

## Overview

The services crate uses `mockall` for generating mock implementations of service traits. Two mock patterns exist:

1. **`#[automock]` on traits** -- automatic mock generation for simple service traits
2. **`mockall::mock!` macro** -- manual mock for composite traits like `DbService`

## MockDbService (Composite Mock)

`DbService` is a composite of multiple repository traits (`DbCore`, `ModelRepository`, `AccessRepository`, `TokenRepository`). The mock is defined manually in `test_utils/db.rs`:

```rust
use crate::test_utils::MockDbService;

let mut mock_db = MockDbService::new();
mock_db
  .expect_list_api_tokens()
  .with(eq("user123"), eq(1), eq(10))
  .returning(|_, _, _| Ok((vec![/* ... */], 0)));
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
  .expect_create_api_token()
  .times(1)
  .returning(|_| Err(DbError::SqlxError(SqlxError(sqlx::Error::RowNotFound))));

// Never expect a method to be called
mock_db
  .expect_delete_api_token()
  .never();
```

## Automock Traits

Most service traits use `#[cfg_attr(test, mockall::automock)]` for automatic mock generation:

```rust
// In the service definition:
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait McpService: Send + Sync + std::fmt::Debug {
  async fn list(&self, tenant_id: &str, user_id: &str) -> Result<Vec<McpEntity>, McpError>;
}

// In tests:
use crate::mcps::MockMcpService;

let mut mock_mcp = MockMcpService::new();
mock_mcp
  .expect_list()
  .returning(|_, _| Ok(vec![/* ... */]));
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
