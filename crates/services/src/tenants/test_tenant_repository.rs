use crate::AppStatus;
use crate::{
  tenants::TenantRepository,
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID},
  Tenant,
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_test_tenant(id: &str, client_id: &str, now: chrono::DateTime<chrono::Utc>) -> Tenant {
  Tenant {
    id: id.to_string(),
    client_id: client_id.to_string(),
    client_secret: "test-secret".to_string(),
    name: format!("Tenant {id}"),
    description: None,
    status: AppStatus::Ready,
    created_by: Some(TEST_USER_ID.to_string()),
    created_at: now,
    updated_at: now,
  }
}

fn make_test_tenant_no_membership(
  id: &str,
  client_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> Tenant {
  Tenant {
    id: id.to_string(),
    client_id: client_id.to_string(),
    client_secret: "test-secret".to_string(),
    name: format!("Tenant {id}"),
    description: None,
    status: AppStatus::ResourceAdmin,
    created_by: None,
    created_at: now,
    updated_at: now,
  }
}

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
    .create_tenant(
      "test-client-id",
      secret,
      "Test App",
      None,
      &AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  assert!(!created.id.is_empty());
  assert_eq!("test-client-id", created.client_id);
  assert_eq!(secret, created.client_secret);
  assert_eq!(AppStatus::ResourceAdmin, created.app_status);
  assert_eq!(ctx.now, created.created_at);
  assert_eq!(ctx.now, created.updated_at);

  let row = ctx.service.get_tenant().await?.expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("test-client-id", row.client_id);
  assert_eq!(secret, row.client_secret);
  assert_eq!(AppStatus::ResourceAdmin, row.app_status);

  Ok(())
}

// =========================================================================
// create_tenant with Ready status auto-creates membership
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_tenant_ready_creates_membership(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let created = ctx
    .service
    .create_tenant(
      "test-client-id",
      "test-secret",
      "Test App",
      None,
      &AppStatus::Ready,
      Some(TEST_USER_ID.to_string()),
    )
    .await?;
  assert_eq!(AppStatus::Ready, created.app_status);
  assert_eq!(Some(TEST_USER_ID.to_string()), created.created_by);

  // Membership should have been auto-created
  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  Ok(())
}

// =========================================================================
// create_tenant with Ready status but no created_by returns validation error
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_tenant_ready_without_created_by_fails(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let result = ctx
    .service
    .create_tenant(
      "test-client-id",
      "test-secret",
      "Test App",
      None,
      &AppStatus::Ready,
      None,
    )
    .await;

  let err = result.unwrap_err();
  assert_eq!("db_error-validation_error", err.code());

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
    .create_tenant(
      "client-1",
      "secret-1",
      "Client 1",
      None,
      &AppStatus::ResourceAdmin,
      None,
    )
    .await?;

  let row = ctx
    .service
    .get_tenant_by_id(&created.id)
    .await?
    .expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("client-1", row.client_id);
  assert_eq!("secret-1", row.client_secret);
  assert_eq!(AppStatus::ResourceAdmin, row.app_status);

  // Non-existent id returns None
  let result = ctx.service.get_tenant_by_id("no-such-id").await?;
  assert!(result.is_none());

  Ok(())
}

// =========================================================================
// set_tenant_ready: atomically sets status, created_by, and membership
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_set_tenant_ready(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let created = ctx
    .service
    .create_tenant(
      "client-1",
      "secret-1",
      "Client 1",
      None,
      &AppStatus::ResourceAdmin,
      None,
    )
    .await?;

  ctx
    .service
    .set_tenant_ready(&created.id, TEST_USER_ID)
    .await?;

  let row = ctx
    .service
    .get_tenant_by_id(&created.id)
    .await?
    .expect("row should exist");
  assert_eq!(AppStatus::Ready, row.app_status);
  assert_eq!(Some(TEST_USER_ID.to_string()), row.created_by);

  // Membership should have been created atomically
  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  Ok(())
}

// =========================================================================
// set_tenant_ready not found -> error
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_set_tenant_ready_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let result = ctx
    .service
    .set_tenant_ready("nonexistent-tenant", TEST_USER_ID)
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
    .create_tenant(
      "client-1",
      "secret-1",
      "Client 1",
      None,
      &AppStatus::ResourceAdmin,
      None,
    )
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
    .create_tenant(
      "client-1",
      "secret-1",
      "Client 1",
      None,
      &AppStatus::ResourceAdmin,
      None,
    )
    .await?;

  let row = ctx
    .service
    .get_tenant_by_client_id("client-1")
    .await?
    .expect("row should exist");
  assert_eq!(created.id, row.id);
  assert_eq!("client-1", row.client_id);
  assert_eq!("secret-1", row.client_secret);
  assert_eq!(AppStatus::ResourceAdmin, row.app_status);

  // Non-existent client_id returns None
  let result = ctx
    .service
    .get_tenant_by_client_id("no-such-client")
    .await?;
  assert!(result.is_none());

  Ok(())
}

// =========================================================================
// upsert_tenant_user + has_tenant_memberships roundtrip
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_upsert_tenant_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  // Use a tenant without created_by so no auto-membership is created
  let tenant = make_test_tenant_no_membership(TEST_TENANT_ID, "client-a", ctx.now);
  ctx.service.create_tenant_test(&tenant).await?;

  // No memberships initially
  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(false, has);

  // Upsert membership
  ctx
    .service
    .upsert_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  // Upsert again (idempotent — updates updated_at)
  ctx
    .service
    .upsert_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  Ok(())
}

// =========================================================================
// delete_tenant_user
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_tenant_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let tenant = make_test_tenant(TEST_TENANT_ID, "client-a", ctx.now);
  ctx.service.create_tenant_test(&tenant).await?;

  // Membership already exists from create_tenant_test (created_by is set)
  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  ctx
    .service
    .delete_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(false, has);

  // Deleting non-existent membership is idempotent
  ctx
    .service
    .delete_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  Ok(())
}

// =========================================================================
// list_user_tenants
// =========================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_user_tenants(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let tenant_a = make_test_tenant(TEST_TENANT_ID, "client-a", ctx.now);
  let tenant_b = make_test_tenant(TEST_TENANT_B_ID, "client-b", ctx.now);

  ctx.service.create_tenant_test(&tenant_a).await?;
  ctx.service.create_tenant_test(&tenant_b).await?;

  // Both tenants have auto-created membership from create_tenant_test
  let tenants = ctx.service.list_user_tenants(TEST_USER_ID).await?;
  assert_eq!(2, tenants.len());

  // User with no memberships returns empty
  let empty = ctx.service.list_user_tenants("no-such-user").await?;
  assert_eq!(0, empty.len());

  Ok(())
}
