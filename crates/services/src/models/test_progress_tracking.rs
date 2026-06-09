use crate::models::progress_tracking::DatabaseProgress;
use crate::models::{HubFile, Repo};
use crate::Progress;
use crate::{
  models::{DownloadRepository, DownloadRequestEntity},
  test_utils::{test_db_service, test_hf_service, TestDbService, TestHfService, TEST_TENANT_ID},
  HubService,
};
use anyhow_trace::anyhow_trace;
use hf_hub::api::tokio::Progress as HfProgress;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::{sync::Arc, time::Duration};

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

  let request = DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "test.gguf", now);
  db_service.create_download_request(&request).await?;

  let mut progress = Progress::Database(DatabaseProgress::new(
    db_service.clone(),
    request.tenant_id.clone(),
    request.id.clone(),
  ));

  progress.init(4096, "test.gguf").await;

  let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
  assert!(event_received, "Timed out waiting for init update");

  // Updates are batched every 3s, so these produce no immediate database events.
  progress.update(1024).await;
  progress.update(1024).await;
  progress.update(2048).await;

  progress.finish().await;

  // finish() always syncs immediately.
  let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
  assert!(event_received, "Timed out waiting for finish update");

  let retrieved = db_service
    .get_download_request(TEST_TENANT_ID, &request.id)
    .await?;
  assert!(retrieved.is_some());

  let retrieved = retrieved.unwrap();
  assert_eq!(retrieved.downloaded_bytes, 4096);
  assert_eq!(retrieved.total_bytes, Some(4096));
  assert!(retrieved.started_at.is_some());
  assert_eq!(retrieved.updated_at, db_service.now()); // frozen time

  Ok(())
}

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
  let now = db_service.now();
  let db_service = Arc::new(db_service);

  let request = DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "test.gguf", now);
  db_service.create_download_request(&request).await?;

  test_hf_service
    .inner_mock
    .expect_download()
    .times(1)
    .returning(move |_, _, _, _| Ok(HubFile::testalias()));

  let progress = Progress::Database(DatabaseProgress::new(
    db_service.clone(),
    request.tenant_id.clone(),
    request.id.clone(),
  ));
  let result = test_hf_service
    .download(&Repo::testalias(), "test.gguf", None, Some(progress))
    .await;

  assert!(result.is_ok());
  Ok(())
}
