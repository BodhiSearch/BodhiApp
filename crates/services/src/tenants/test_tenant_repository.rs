use crate::AppStatus;
use crate::{
  tenants::TenantRepository,
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// =========================================================================
// create + get roundtrip (verify encryption/decryption)
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let secret = "super-secret-value-123";

  let created = ctx
    .service
    .create_tenant("test-client-id", secret, &AppStatus::Ready, None)
    .await?;
  assert!(!created.id.is_empty());
  assert_eq!("test-client-id", created.client_id);
  assert_eq!(secret, created.client_secret);
  assert_eq!(AppStatus::Ready, created.app_status);
  assert_eq!(ctx.now, created.created_at);
  assert_eq!(ctx.now, created.updated_at);

  let row = ctx.service.get_tenant().await?.expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("test-client-id", row.client_id);
  assert_eq!(secret, row.client_secret);
  assert_eq!(AppStatus::Ready, row.app_status);

  Ok(())
}

// =========================================================================
// get when empty -> None
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_tenant_empty(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_tenant().await?;
  assert_eq!(None, result.map(|_| ()));
  Ok(())
}

// =========================================================================
// get_tenant_by_id
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_tenant_by_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let created = ctx
    .service
    .create_tenant("client-1", "secret-1", &AppStatus::Ready, None)
    .await?;

  let row = ctx
    .service
    .get_tenant_by_id(&created.id)
    .await?
    .expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("client-1", row.client_id);
  assert_eq!("secret-1", row.client_secret);
  assert_eq!(AppStatus::Ready, row.app_status);

  // Non-existent id returns None
  let result = ctx.service.get_tenant_by_id("no-such-id").await?;
  assert!(result.is_none());

  Ok(())
}

// =========================================================================
// update_tenant_status
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_tenant_status(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_tenant("client-1", "secret-1", &AppStatus::Setup, None)
    .await?;

  ctx
    .service
    .update_tenant_status("client-1", &AppStatus::Ready)
    .await?;

  let row = ctx.service.get_tenant().await?.expect("row should exist");
  assert_eq!(AppStatus::Ready, row.app_status);

  Ok(())
}

// =========================================================================
// update_tenant_status not found -> error
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_status_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let result = ctx
    .service
    .update_tenant_status("nonexistent-client", &AppStatus::Ready)
    .await;

  let err = result.unwrap_err();
  assert_eq!("db_error-item_not_found", err.code());

  Ok(())
}

// =========================================================================
// delete_tenant
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_tenant(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_tenant("client-1", "secret-1", &AppStatus::Ready, None)
    .await?;

  ctx.service.delete_tenant("client-1").await?;

  let result = ctx.service.get_tenant().await?;
  assert!(result.is_none());

  Ok(())
}

// =========================================================================
// get_tenant_by_client_id
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_tenant_by_client_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let created = ctx
    .service
    .create_tenant("client-1", "secret-1", &AppStatus::Ready, None)
    .await?;

  let row = ctx
    .service
    .get_tenant_by_client_id("client-1")
    .await?
    .expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("client-1", row.client_id);
  assert_eq!("secret-1", row.client_secret);
  assert_eq!(AppStatus::Ready, row.app_status);

  // Non-existent client_id returns None
  let result = ctx
    .service
    .get_tenant_by_client_id("no-such-client")
    .await?;
  assert!(result.is_none());

  Ok(())
}
