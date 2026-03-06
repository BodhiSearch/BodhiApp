use crate::models::{DownloadRepository, DownloadRequestEntity};
use crate::test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_download_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let dl_a =
    DownloadRequestEntity::new_pending(TEST_TENANT_ID, "test/repo-a", "file_a.gguf", ctx.now);
  let dl_b =
    DownloadRequestEntity::new_pending(TEST_TENANT_B_ID, "test/repo-b", "file_b.gguf", ctx.now);

  ctx.service.create_download_request(&dl_a).await?;
  ctx.service.create_download_request(&dl_b).await?;

  // Listing downloads in tenant A should only return dl_a
  let (list_a, total_a) = ctx
    .service
    .list_download_requests(TEST_TENANT_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, list_a.len());
  assert_eq!(TEST_TENANT_ID, list_a[0].tenant_id);

  // Listing downloads in tenant B should only return dl_b
  let (list_b, total_b) = ctx
    .service
    .list_download_requests(TEST_TENANT_B_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, list_b.len());
  assert_eq!(TEST_TENANT_B_ID, list_b[0].tenant_id);

  // Getting dl_b by ID under tenant A should return None
  let cross = ctx
    .service
    .get_download_request(TEST_TENANT_ID, &dl_b.id)
    .await?;
  assert_eq!(None, cross);

  Ok(())
}
