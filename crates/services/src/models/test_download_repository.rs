use crate::{
  models::{
    DefaultDownloadService, DownloadRepository, DownloadRequestEntity, DownloadService,
    DownloadServiceError, DownloadStatus,
  },
  test_utils::{sea_context, setup_env, FrozenTimeService, TEST_TENANT_ID},
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::sync::Arc;

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_download_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let request =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  let fetched = ctx
    .service
    .get_download_request(TEST_TENANT_ID, &request.id)
    .await?;
  assert!(fetched.is_some());
  assert_eq!(request, fetched.unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_download_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut request =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  request.status = DownloadStatus::Completed;
  request.total_bytes = Some(1000000);
  request.downloaded_bytes = 1000000;
  request.started_at = Some(ctx.now);
  request.updated_at = ctx.now + chrono::Duration::hours(1);
  ctx.service.update_download_request(&request).await?;

  let fetched = ctx
    .service
    .get_download_request(TEST_TENANT_ID, &request.id)
    .await?
    .unwrap();
  assert_eq!(DownloadStatus::Completed, fetched.status);
  assert_eq!(Some(1000000), fetched.total_bytes);
  assert_eq!(1000000, fetched.downloaded_bytes);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_download_requests(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  for i in 0..5 {
    let request = DownloadRequestEntity::new_pending(
      TEST_TENANT_ID,
      "test/repo",
      &format!("file_{}.gguf", i),
      ctx.now,
    );
    ctx.service.create_download_request(&request).await?;
  }
  let (items, total) = ctx
    .service
    .list_download_requests(TEST_TENANT_ID, 1, 3)
    .await?;
  assert_eq!(3, items.len());
  assert_eq!(5, total);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_find_download_by_repo_filename(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let r1 = DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "file.gguf", ctx.now);
  let r2 = DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "file.gguf", ctx.now);
  let r3 = DownloadRequestEntity::new_pending(TEST_TENANT_ID, "other/repo", "file.gguf", ctx.now);
  ctx.service.create_download_request(&r1).await?;
  ctx.service.create_download_request(&r2).await?;
  ctx.service.create_download_request(&r3).await?;

  let results = ctx
    .service
    .find_download_request_by_repo_filename(TEST_TENANT_ID, "test/repo", "file.gguf")
    .await?;
  assert_eq!(2, results.len());
  Ok(())
}

/// Builds a DefaultDownloadService over the test DB so we exercise the archive/retry
/// business logic (status guards) on top of the real repository.
fn download_service(ctx: &crate::test_utils::SeaTestContext) -> DefaultDownloadService {
  DefaultDownloadService::new(
    Arc::new(ctx.service.clone()),
    Arc::new(FrozenTimeService::default()),
  )
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_excludes_archived(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let svc = download_service(&ctx);

  // Two completed downloads; archive one — list must show only the other.
  let mut a =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "a.gguf", ctx.now);
  a.status = DownloadStatus::Completed;
  let mut b =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "b.gguf", ctx.now);
  b.status = DownloadStatus::Completed;
  ctx.service.create_download_request(&a).await?;
  ctx.service.create_download_request(&b).await?;

  svc.archive(TEST_TENANT_ID, &a.id).await?;

  let resp = svc.list(TEST_TENANT_ID, 1, 30).await?;
  assert_eq!(1, resp.total);
  assert_eq!(1, resp.data.len());
  assert_eq!(b.id, resp.data[0].id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_archive_rejects_active_download(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let svc = download_service(&ctx);

  // Pending + started_at set = actively downloading → cannot archive.
  let mut active =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "active.gguf", ctx.now);
  active.started_at = Some(ctx.now);
  ctx.service.create_download_request(&active).await?;

  let err = svc.archive(TEST_TENANT_ID, &active.id).await.unwrap_err();
  assert_eq!("download_service_error-cannot_archive_active", err.code());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_archive_allows_queued(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let svc = download_service(&ctx);

  // Pending + no started_at = queued → archivable.
  let queued =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "queued.gguf", ctx.now);
  ctx.service.create_download_request(&queued).await?;

  let archived = svc.archive(TEST_TENANT_ID, &queued.id).await?;
  assert!(archived.archived_at.is_some());
  assert_eq!(0, svc.list(TEST_TENANT_ID, 1, 30).await?.total);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_reset_for_retry_failed(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let svc = download_service(&ctx);

  let mut failed =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "failed.gguf", ctx.now);
  failed.status = DownloadStatus::Error;
  failed.error = Some("boom".to_string());
  failed.started_at = Some(ctx.now);
  ctx.service.create_download_request(&failed).await?;

  let reset = svc.reset_for_retry(TEST_TENANT_ID, &failed.id).await?;
  assert_eq!(DownloadStatus::Pending, reset.status);
  assert_eq!(None, reset.error);
  assert_eq!(None, reset.started_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_reset_for_retry_rejects_non_failed(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let svc = download_service(&ctx);

  let mut completed =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo", "done.gguf", ctx.now);
  completed.status = DownloadStatus::Completed;
  ctx.service.create_download_request(&completed).await?;

  let err = svc
    .reset_for_retry(TEST_TENANT_ID, &completed.id)
    .await
    .unwrap_err();
  assert_eq!("download_service_error-not_retryable", err.code());
  assert!(matches!(err, DownloadServiceError::NotRetryable));
  Ok(())
}
