use crate::{
  models::{DownloadRepository, DownloadRequest, DownloadStatus},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// ===== Download Request Tests =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_download_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let request = DownloadRequest::new_pending("test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  let fetched = ctx.service.get_download_request(&request.id).await?;
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
  let mut request = DownloadRequest::new_pending("test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  request.status = DownloadStatus::Completed;
  request.total_bytes = Some(1000000);
  request.downloaded_bytes = 1000000;
  request.started_at = Some(ctx.now);
  request.updated_at = ctx.now + chrono::Duration::hours(1);
  ctx.service.update_download_request(&request).await?;

  let fetched = ctx
    .service
    .get_download_request(&request.id)
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
    let request = DownloadRequest::new_pending("test/repo", &format!("file_{}.gguf", i), ctx.now);
    ctx.service.create_download_request(&request).await?;
  }
  let (items, total) = ctx.service.list_download_requests(1, 3).await?;
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
  let r1 = DownloadRequest::new_pending("test/repo", "file.gguf", ctx.now);
  let r2 = DownloadRequest::new_pending("test/repo", "file.gguf", ctx.now);
  let r3 = DownloadRequest::new_pending("other/repo", "file.gguf", ctx.now);
  ctx.service.create_download_request(&r1).await?;
  ctx.service.create_download_request(&r2).await?;
  ctx.service.create_download_request(&r3).await?;

  let results = ctx
    .service
    .find_download_request_by_repo_filename("test/repo", "file.gguf")
    .await?;
  assert_eq!(2, results.len());
  Ok(())
}
