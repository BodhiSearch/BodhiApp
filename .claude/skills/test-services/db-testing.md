# Database Testing Patterns

## TestDbService Fixture

The `test_db_service` rstest fixture provides a real SQLite database in a temporary directory with:
- Automatic migration
- `FrozenTimeService` for deterministic timestamps
- Event broadcasting for operation verification
- Encryption key for testing encrypted storage

### Basic Usage

```rust
use crate::test_utils::{test_db_service, TestDbService};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_db_create_and_fetch(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let request = DownloadRequest::new_pending("test/repo", "test_file.gguf", now);
  service.create_download_request(&request).await?;

  let fetched = service.get_download_request(&request.id).await?;
  assert!(fetched.is_some());
  assert_eq!(request, fetched.unwrap());
  Ok(())
}
```

### Key Points

- Always use `#[awt]` and `#[future]` with the `test_db_service` fixture (it is async)
- Use `#[from(test_db_service)]` to explicitly name the fixture source
- Call `service.now()` to get the frozen timestamp for constructing expected values
- The fixture creates a fresh SQLite DB per test -- no cross-test contamination

## FrozenTimeService

`FrozenTimeService` captures `Utc::now()` at creation (with nanoseconds zeroed) and always returns that same value:

```rust
pub struct FrozenTimeService(DateTime<Utc>);

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0  // Always returns the same frozen timestamp
  }
}
```

Use `service.now()` in tests to get the frozen time for expected value construction:

```rust
let now = service.now();
let expected = UserAccessRequest {
  created_at: now,
  updated_at: now,
  status: UserAccessRequestStatus::Pending,
  // ...
};
assert_eq!(expected, actual);
```

## Event Broadcasting

`TestDbService` broadcasts events for every database operation via `tokio::sync::broadcast`. Use `subscribe()` to verify operation ordering:

```rust
let mut rx = db_service.subscribe();
let db_service = Arc::new(db_service);

// Perform operations...
db_service.create_download_request(&request).await?;

// Wait for the event
let event_received = loop {
  tokio::select! {
    event = rx.recv() => {
      match event {
        Ok(e) if e == "create_download_request" => break true,
        _ => continue
      }
    }
    _ = tokio::time::sleep(Duration::from_millis(100)) => break false
  }
};
assert!(event_received, "Timed out waiting for create event");
```

Event names match the method names: `"create_download_request"`, `"update_download_request"`, `"get_download_request"`, `"migrate"`, etc.

## Shared TempDir Pattern

When multiple services need the same temp directory, use `test_db_service_with_temp_dir`:

```rust
use crate::test_utils::test_db_service_with_temp_dir;
use std::sync::Arc;
use tempfile::TempDir;

let shared_temp_dir = Arc::new(TempDir::new()?);
let db_service = test_db_service_with_temp_dir(shared_temp_dir.clone()).await;
// shared_temp_dir can be passed to other services too
```

## Testing Repository Traits

`TestDbService` implements all repository traits (`ModelRepository`, `AccessRepository`, `TokenRepository`, `ToolsetRepository`) by delegating to a real `SqliteDbService`. This means tests exercise real SQL queries:

```rust
// ModelRepository methods
service.create_download_request(&request).await?;
service.get_download_request(&id).await?;
service.list_download_requests(page, page_size).await?;
service.create_api_model_alias(&alias, api_key).await?;

// AccessRepository methods
service.insert_pending_request(username, user_id).await?;
service.list_pending_requests(page, per_page).await?;
service.update_request_status(id, status, reviewer).await?;

// TokenRepository methods
service.create_api_token(&mut token).await?;
service.list_api_tokens(user_id, page, per_page).await?;
service.get_api_token_by_prefix(prefix).await?;

// ToolsetRepository methods
service.create_toolset(&row).await?;
service.list_toolsets(user_id).await?;
```

## Pagination Testing

Test pagination by inserting multiple records and verifying page boundaries:

```rust
// Insert 3 records
for (username, user_id) in &test_data {
  service.insert_pending_request(username.clone(), user_id.clone()).await?;
}

// Verify page 1
let (page1, total) = service.list_pending_requests(1, 2).await?;
assert_eq!(2, page1.len());
assert_eq!(3, total);

// Verify page 2
let (page2, total) = service.list_pending_requests(2, 2).await?;
assert_eq!(1, page2.len());
assert_eq!(3, total);
```
