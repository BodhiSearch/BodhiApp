# Advanced Testing Patterns

## Concurrency Testing

The `ConcurrencyService` tests verify that per-key locking serializes operations on the same key while allowing parallelism across different keys.

### Same-Key Sequential Execution

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_sequential_execution_same_key() {
  let service = Arc::new(LocalConcurrencyService::new());
  let counter = Arc::new(AtomicU32::new(0));

  let counter1 = Arc::clone(&counter);
  let service1 = Arc::clone(&service);
  let handle1 = tokio::spawn(async move {
    let _ = service1
      .with_lock_auth(
        "test_key",
        Box::new(move || {
          Box::pin(async move {
            let val = counter1.load(Ordering::SeqCst);
            sleep(Duration::from_millis(10)).await;
            counter1.store(val + 1, Ordering::SeqCst);
            Ok(("token1".to_string(), None))
          })
        }),
      )
      .await;
  });

  // ... spawn handle2 with same "test_key" ...

  handle1.await.unwrap();
  handle2.await.unwrap();

  // Both increments succeed due to locking (no lost updates)
  assert_eq!(2, counter.load(Ordering::SeqCst));
}
```

### Different-Key Parallel Execution

```rust
#[tokio::test]
async fn test_concurrent_execution_different_keys() {
  let service = Arc::new(LocalConcurrencyService::new());
  let start_time = std::time::Instant::now();

  // Spawn tasks with different keys -- they run concurrently
  let handle1 = tokio::spawn(/* with_lock_auth("key1", 50ms sleep) */);
  let handle2 = tokio::spawn(/* with_lock_auth("key2", 50ms sleep) */);

  handle1.await.unwrap();
  handle2.await.unwrap();

  // Should complete in ~50ms (parallel), not ~100ms (sequential)
  assert!(start_time.elapsed() < Duration::from_millis(90));
}
```

## Progress Tracking Tests

Progress tracking tests use the `TestDbService` event broadcasting to verify async database updates.

### wait_for_event! Macro

```rust
macro_rules! wait_for_event {
  ($rx:expr, $event_name:expr, $timeout:expr) => {{
    loop {
      tokio::select! {
        event = $rx.recv() => {
          match event {
            Ok(e) if e == $event_name => break true,
            _ => continue
          }
        }
        _ = tokio::time::sleep($timeout) => break false
      }
    }
  }};
}
```

### DatabaseProgress Integration Test

```rust
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_database_progress_integration(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let mut rx = db_service.subscribe();
  let db_service = Arc::new(db_service);

  // Create initial download request
  let request = DownloadRequest::new_pending("test/repo", "test.gguf", now);
  db_service.create_download_request(&request).await?;

  // Create progress tracker
  let mut progress = Progress::Database(DatabaseProgress::new(
    db_service.clone(),
    request.id.clone(),
  ));

  // Simulate hf-hub calling init/update/finish
  progress.init(4096, "test.gguf").await;
  let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
  assert!(event_received);

  progress.update(1024).await;
  progress.update(3072).await;
  progress.finish().await;

  // Verify final state
  let retrieved = db_service.get_download_request(&request.id).await?.unwrap();
  assert_eq!(4096, retrieved.downloaded_bytes);
  assert_eq!(Some(4096), retrieved.total_bytes);
  Ok(())
}
```

## Setting Change Notification Tests

Tests verify that `SettingService` notifies listeners when settings change.

### Parameterized Notification Test

```rust
#[derive(Debug)]
enum NotificationOperation {
  OverrideSetting,
  DeleteSetting,
  SetWithEnvOverride,
  SetDefault,
}

#[rstest]
#[case::override_existing(
  NotificationOperation::OverrideSetting,
  None,                // env_value
  Some("old_value"),   // initial_file_value
  None,                // default_value
  Some("new_value"),   // new_value
  Some(("old_value", SettingSource::SettingsFile, "new_value", SettingSource::SettingsFile))
)]
#[case::no_notification_for_defaults(
  NotificationOperation::SetDefault,
  None, Some("file_value"), None, Some("default_value"),
  None  // No notification expected
)]
fn test_change_notifications(
  temp_dir: TempDir,
  #[case] operation: NotificationOperation,
  #[case] env_value: Option<&str>,
  #[case] initial_file_value: Option<&str>,
  #[case] default_value: Option<&str>,
  #[case] new_value: Option<&str>,
  #[case] expected_notification: Option<(Option<&str>, SettingSource, Option<&str>, SettingSource)>,
) -> anyhow::Result<()> {
  // Setup service with initial state...

  let mut mock_listener = MockSettingsChangeListener::default();
  match expected_notification {
    Some((old_val, old_source, new_val, new_source)) => {
      mock_listener
        .expect_on_change()
        .with(eq("TEST_KEY"), /* old */, eq(old_source), /* new */, eq(new_source))
        .times(1)
        .return_once(|_, _, _, _, _| ());
    }
    None => {
      mock_listener.expect_on_change().never();
    }
  }
  service.add_listener(Arc::new(mock_listener));

  // Perform operation...
  Ok(())
}
```

## Parameterized Tests with `#[case]`

Use `#[case]` for testing multiple scenarios with the same test body:

### Setting Precedence Test

```rust
#[rstest]
#[case::system_settings_cannot_be_overridden(
  "TEST_SYSTEM_KEY", Some("cmdline"), Some("env"), Some("file"), Some("default"),
  "system_value", SettingSource::System
)]
#[case::command_line_highest_priority(
  "TEST_KEY", Some("cmdline"), Some("env"), Some("file"), Some("default"),
  "cmdline", SettingSource::CommandLine
)]
#[case::environment_override(
  "TEST_KEY", None, Some("env"), Some("file"), Some("default"),
  "env", SettingSource::Environment
)]
fn test_settings_precedence(
  temp_dir: TempDir,
  #[case] key: &str,
  #[case] cmdline_value: Option<&str>,
  #[case] env_value: Option<&str>,
  #[case] file_value: Option<&str>,
  #[case] default_value: Option<&str>,
  #[case] expected_value: &str,
  #[case] expected_source: SettingSource,
) -> anyhow::Result<()> {
  // ... test logic using all case parameters ...
  Ok(())
}
```

## HubService Mock with Progress

Testing services that accept progress callbacks:

```rust
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_hub_service_with_database_progress(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
  #[from(test_hf_service)] mut test_hf_service: TestHfService,
) -> anyhow::Result<()> {
  // Setup mock to accept any progress parameter
  test_hf_service
    .inner_mock
    .expect_download()
    .times(1)
    .returning(move |_, _, _, _| Ok(HubFile::testalias()));

  // ... create service, call download with progress ...
  Ok(())
}
```

## EnvWrapperStub

For testing environment-dependent behavior:

```rust
use crate::test_utils::EnvWrapperStub;

let env_stub = EnvWrapperStub::new(maplit::hashmap! {
  "BODHI_HOME".to_string() => temp_dir.path().display().to_string(),
  "TEST_KEY".to_string() => "test_value".to_string(),
});
let service = DefaultSettingService::new(Arc::new(env_stub), path, vec![]);
```

## Helper Function Pattern

For test data construction used across multiple tests, define module-level helper functions:

```rust
fn test_toolset_row(id: &str, user_id: &str, name: &str) -> ToolsetRow {
  ToolsetRow {
    id: id.to_string(),
    user_id: user_id.to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    name: name.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: Some("encrypted".to_string()),
    salt: Some("salt".to_string()),
    nonce: Some("nonce".to_string()),
    created_at: 1700000000,
    updated_at: 1700000000,
  }
}
```

Keep these at the top of `mod tests`, not inside individual test functions.
