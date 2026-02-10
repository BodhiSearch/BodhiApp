---
name: test-services
description: >
  Use when writing or migrating unit tests for the services crate.
  Covers canonical test patterns, DB fixture setup, mockito HTTP mocking,
  mockall service mocking, error assertions, concurrency tests, and
  notification listener tests. Examples: "write tests for a new service method",
  "migrate old tests to canonical pattern", "add coverage for error paths".
---

# services Crate Unit Test Skill

Write and migrate unit tests for the `services` crate following uniform conventions.

## Quick Reference

Every service test follows one of two shapes depending on sync vs async:

### Async test (database or HTTP)

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<service>_<scenario>() -> anyhow::Result<()> {
  // 1. Setup service with mocks or real DB
  // 2. Call service method
  // 3. Assert result
  Ok(())
}
```

### Async test with database fixture

```rust
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_<service>_<scenario>(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  // ... test logic ...
  Ok(())
}
```

### Sync test (no async, no database)

```rust
#[rstest]
fn test_<service>_<scenario>() -> anyhow::Result<()> {
  // 1. Setup service with mocks
  // 2. Call service method
  // 3. Assert result
  Ok(())
}
```

## Core Rules

1. **Annotations (async)**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` on every async test. Add `#[awt]` ONLY when `#[future]` fixture params are used.
2. **Annotations (sync)**: `#[rstest]` only. Do NOT add `#[anyhow_trace]` on sync tests.
3. **Naming**: `test_<service_name>_<scenario>` (e.g. `test_db_service_create_download_request`)
4. **Module**: `mod tests` (not `mod test`)
5. **Return**: Always `-> anyhow::Result<()>` with `Ok(())` at end
6. **Errors**: Use `?` not `.unwrap()`. Use `.expect("msg")` only in non-`?` contexts (closures, Option chains)
7. **Assertions**: `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;`
8. **Error codes**: Assert via `.code()` method. Codes are `enum_name-variant_name` in snake_case.
9. **Transparent errors**: Errors with `#[error(transparent)]` delegate to inner error code (e.g., `DbError::SqlxError` -> `"sqlx_error"`, NOT `"db_error-sqlx_error"`)
10. **No `use super::*`**: Use explicit imports in test modules to avoid refactoring issues.

## Error Code Convention

Error codes are auto-generated from enum name + variant name in snake_case:

```rust
// AuthServiceError::AuthServiceApiError -> "auth_service_error-auth_service_api_error"
// DataServiceError::AliasNotExists     -> "data_service_error-alias_not_exists"
// ToolsetError::ToolsetNotFound        -> "toolset_error-toolset_not_found"
```

**Transparent error delegation**: When a variant uses `#[error(transparent)]`:
```rust
pub enum DbError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),  // .code() returns "sqlx_error", NOT "db_error-sqlx_error"
}
```

Assert error codes like:
```rust
let err = result.unwrap_err();
assert_eq!("auth_service_error-auth_service_api_error", err.code());
```

Or use `matches!` for error variant + field checks:
```rust
assert!(matches!(
  result.unwrap_err(),
  AiApiServiceError::PromptTooLong { max_length: 30, actual_length: 31 }
));
```

## When to Use `#[awt]`

Use `#[awt]` ONLY when test parameters use `#[future]`:

```rust
// YES - has #[future] fixture param
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_something(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> { ... }

// NO - no #[future] params
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_something_else() -> anyhow::Result<()> { ... }
```

## Pattern Files

For detailed patterns with full code examples, see:

- **[db-testing.md](db-testing.md)** -- TestDbService fixture, FrozenTimeService, real SQLite tests
- **[api-testing.md](api-testing.md)** -- mockito patterns for HTTP service testing
- **[mock-patterns.md](mock-patterns.md)** -- mockall setup, MockDbService, expectation-driven tests
- **[advanced.md](advanced.md)** -- Concurrency, progress tracking, setting notifications, parameterized tests

## Standard Imports

```rust
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;
```

Additional imports vary by test type:
- **DB tests**: `use crate::test_utils::{test_db_service, TestDbService};`
- **Mock tests**: `use crate::test_utils::MockDbService;` + `use mockall::predicate::eq;`
- **HTTP tests**: `use mockito::{Matcher, Server};` + `use serde_json::json;`

## Migration Checklist

When migrating existing tests to the canonical pattern:

- [ ] Add `use pretty_assertions::assert_eq;`
- [ ] Add `use anyhow_trace::anyhow_trace;` (async tests only)
- [ ] Ensure correct annotation order: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` (async) or just `#[rstest]` (sync)
- [ ] Add `#[awt]` only if `#[future]` fixture params exist
- [ ] Change return type to `-> anyhow::Result<()>` with `Ok(())` at end
- [ ] Replace `.unwrap()` with `?` (or `.expect()` in closures)
- [ ] Convert error message assertions to error code assertions via `.code()`
- [ ] Verify `assert_eq!(expected, actual)` order
- [ ] Replace `use super::*` with explicit imports
- [ ] Module name is `mod tests` (not `mod test`)
- [ ] Remove `#[allow(unused)]` or dead helper code

## When NOT to Use This Skill

- Route handler tests in `routes_app` (use the `test-routes-app` skill instead)
- Integration tests in `routes_all`
- Frontend/UI tests (use the `playwright` skill instead)
- Tests in `objs` crate (different patterns, no service infrastructure)
