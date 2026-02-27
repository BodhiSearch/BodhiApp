use crate::{
  db::AppInstanceRepository,
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use objs::{AppError, AppStatus};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// =========================================================================
// upsert + get roundtrip (verify encryption/decryption)
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_upsert_and_get_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let secret = "super-secret-value-123";

  ctx
    .service
    .upsert_app_instance("test-client-id", secret, &AppStatus::Ready)
    .await?;

  let row = ctx
    .service
    .get_app_instance()
    .await?
    .expect("row should exist");
  assert_eq!("test-client-id", row.client_id);
  assert_eq!(secret, row.client_secret);
  assert_eq!(AppStatus::Ready, row.app_status);
  assert_eq!(ctx.now, row.created_at);
  assert_eq!(ctx.now, row.updated_at);

  Ok(())
}

// =========================================================================
// get when empty -> None
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_get_app_instance_empty(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_app_instance().await?;
  assert_eq!(None, result.map(|_| ()));
  Ok(())
}

// =========================================================================
// upsert update (change secret/status)
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_upsert_update_existing(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .upsert_app_instance("client-1", "secret-1", &AppStatus::Setup)
    .await?;

  let row = ctx
    .service
    .get_app_instance()
    .await?
    .expect("row should exist");
  assert_eq!("secret-1", row.client_secret);
  assert_eq!(AppStatus::Setup, row.app_status);

  // Upsert with same client_id but different secret/status
  ctx
    .service
    .upsert_app_instance("client-1", "new-secret", &AppStatus::Ready)
    .await?;

  let row = ctx
    .service
    .get_app_instance()
    .await?
    .expect("row should exist");
  assert_eq!("client-1", row.client_id);
  assert_eq!("new-secret", row.client_secret);
  assert_eq!(AppStatus::Ready, row.app_status);

  Ok(())
}

// =========================================================================
// update_app_instance_status
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_update_app_instance_status(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .upsert_app_instance("client-1", "secret-1", &AppStatus::Setup)
    .await?;

  ctx
    .service
    .update_app_instance_status("client-1", &AppStatus::Ready)
    .await?;

  let row = ctx
    .service
    .get_app_instance()
    .await?
    .expect("row should exist");
  assert_eq!(AppStatus::Ready, row.app_status);

  Ok(())
}

// =========================================================================
// update_app_instance_status not found -> error
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_update_status_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let result = ctx
    .service
    .update_app_instance_status("nonexistent-client", &AppStatus::Ready)
    .await;

  let err = result.unwrap_err();
  assert_eq!("db_error-item_not_found", err.code());

  Ok(())
}

// =========================================================================
// delete_app_instance
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_delete_app_instance(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .upsert_app_instance("client-1", "secret-1", &AppStatus::Ready)
    .await?;

  ctx.service.delete_app_instance("client-1").await?;

  let result = ctx.service.get_app_instance().await?;
  assert!(result.is_none());

  Ok(())
}

// =========================================================================
// multiple app instances -> MultipleAppInstance error
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_multiple_app_instances_error(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Insert two different client_ids
  ctx
    .service
    .upsert_app_instance("client-one", "secret-one", &AppStatus::Setup)
    .await?;
  ctx
    .service
    .upsert_app_instance("client-two", "secret-two", &AppStatus::Setup)
    .await?;

  let result = ctx.service.get_app_instance().await;
  let err = result.unwrap_err();
  assert_eq!("db_error-multiple_app_instance", err.code());

  Ok(())
}
