use crate::{
  tenants::TenantRepository,
  test_utils::{test_db_service, TestDbService, TEST_CLIENT_ID, TEST_CLIENT_SECRET},
  AppStatus, DefaultTenantService, TenantService,
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn make_service(db: TestDbService) -> DefaultTenantService {
  DefaultTenantService::new(Arc::new(db))
}

// =========================================================================
// get_status: default when no tenant exists
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_status_no_tenant_returns_setup(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let status = svc.get_status("nonexistent").await?;
  assert_eq!(AppStatus::Setup, status);
  Ok(())
}

// =========================================================================
// create_tenant -> get_tenant round-trip
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_tenant_get_tenant_roundtrip(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let tenant = svc
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  assert_eq!(TEST_CLIENT_ID, tenant.client_id);
  assert_eq!(TEST_CLIENT_SECRET, tenant.client_secret);
  assert_eq!(AppStatus::ResourceAdmin, tenant.status);
  // id should be a non-empty ULID string
  assert!(!tenant.id.is_empty());

  let retrieved = svc
    .get_standalone_app()
    .await?
    .expect("tenant should exist");
  assert_eq!(tenant.id, retrieved.id);
  assert_eq!(tenant.client_id, retrieved.client_id);
  assert_eq!(tenant.client_secret, retrieved.client_secret);
  assert_eq!(tenant.status, retrieved.status);
  Ok(())
}

// =========================================================================
// get_tenant: returns None when no tenant exists
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_tenant_no_tenant_returns_none(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let result = svc.get_standalone_app().await?;
  assert!(result.is_none());
  Ok(())
}

// =========================================================================
// get_status returns correct status after create
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_status_after_create(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  svc
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  let tenant = svc
    .get_standalone_app()
    .await?
    .expect("tenant should exist");
  let status = svc.get_status(&tenant.id).await?;
  assert_eq!(AppStatus::ResourceAdmin, status);
  Ok(())
}

// =========================================================================
// set_tenant_ready: atomically sets status to Ready, created_by, and membership
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_set_tenant_ready_changes_status_created_by_and_membership(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let tenant = svc
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  svc.set_tenant_ready(&tenant.id, "test-user-id").await?;
  let retrieved = svc
    .get_standalone_app()
    .await?
    .expect("tenant should exist");
  assert_eq!(AppStatus::Ready, retrieved.status);
  assert_eq!(Some("test-user-id".to_string()), retrieved.created_by);
  // Membership should also have been created
  let has = svc.has_tenant_memberships("test-user-id").await?;
  assert_eq!(true, has);
  Ok(())
}

// =========================================================================
// singleton enforcement: second upsert with different client_id triggers error
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_two_tenants_triggers_multiple_error(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  db.create_tenant(
    "client-one",
    "secret-one",
    "Client One",
    None,
    &AppStatus::Setup,
    None,
  )
  .await?;
  db.create_tenant(
    "client-two",
    "secret-two",
    "Client Two",
    None,
    &AppStatus::Setup,
    None,
  )
  .await?;

  let svc = make_service(db);
  let result = svc.get_standalone_app().await;
  let err = result.unwrap_err();
  assert_eq!("db_error-multiple_tenant", err.code());
  Ok(())
}

// =========================================================================
// repository: encryption round-trip via upsert + get
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_repository_encryption_roundtrip(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let secret = "super-secret-value-123";
  db.create_tenant(
    TEST_CLIENT_ID,
    secret,
    "Test App",
    None,
    &AppStatus::ResourceAdmin,
    None,
  )
  .await?;

  let row = db.get_tenant().await?.expect("row should exist");
  assert_eq!(secret, row.client_secret);
  assert_eq!(TEST_CLIENT_ID, row.client_id);
  assert_eq!(AppStatus::ResourceAdmin, row.app_status);
  Ok(())
}

// =========================================================================
// repository: delete_tenant removes the row
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_tenant(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  db.create_tenant(
    TEST_CLIENT_ID,
    TEST_CLIENT_SECRET,
    "Test App",
    None,
    &AppStatus::ResourceAdmin,
    None,
  )
  .await?;
  db.delete_tenant(TEST_CLIENT_ID).await?;
  let row = db.get_tenant().await?;
  assert!(row.is_none());
  Ok(())
}
