use crate::{
  db::AppInstanceRepository,
  test_utils::{test_db_service, TestDbService, TEST_CLIENT_ID, TEST_CLIENT_SECRET},
  AppInstanceService, AppStatus, DefaultAppInstanceService,
};
use anyhow_trace::anyhow_trace;
use objs::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn make_service(db: TestDbService) -> DefaultAppInstanceService {
  DefaultAppInstanceService::new(Arc::new(db))
}

// =========================================================================
// get_status: default when no instance exists
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_status_no_instance_returns_setup(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let status = svc.get_status().await?;
  assert_eq!(AppStatus::Setup, status);
  Ok(())
}

// =========================================================================
// create_instance -> get_instance round-trip
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_instance_get_instance_roundtrip(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let instance = svc
    .create_instance(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "test-scope",
      AppStatus::ResourceAdmin,
    )
    .await?;
  assert_eq!(TEST_CLIENT_ID, instance.client_id);
  assert_eq!(TEST_CLIENT_SECRET, instance.client_secret);
  assert_eq!("test-scope", instance.scope);
  assert_eq!(AppStatus::ResourceAdmin, instance.status);

  let retrieved = svc.get_instance().await?.expect("instance should exist");
  assert_eq!(instance.client_id, retrieved.client_id);
  assert_eq!(instance.client_secret, retrieved.client_secret);
  assert_eq!(instance.scope, retrieved.scope);
  assert_eq!(instance.status, retrieved.status);
  Ok(())
}

// =========================================================================
// get_instance: returns None when no instance exists
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_instance_no_instance_returns_none(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let result = svc.get_instance().await?;
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
    .create_instance(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "scope",
      AppStatus::ResourceAdmin,
    )
    .await?;
  let status = svc.get_status().await?;
  assert_eq!(AppStatus::ResourceAdmin, status);
  Ok(())
}

// =========================================================================
// update_status: updates from ResourceAdmin -> Ready
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_status_changes_status(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  svc
    .create_instance(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "scope",
      AppStatus::ResourceAdmin,
    )
    .await?;
  svc.update_status(&AppStatus::Ready).await?;
  let status = svc.get_status().await?;
  assert_eq!(AppStatus::Ready, status);
  Ok(())
}

// =========================================================================
// update_status: error when no instance exists
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_status_no_instance_returns_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let svc = make_service(db);
  let result = svc.update_status(&AppStatus::Ready).await;
  let err = result.unwrap_err();
  assert_eq!("app_instance_error-not_found", err.code());
  Ok(())
}

// =========================================================================
// singleton enforcement: second upsert with different client_id triggers error
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_two_instances_triggers_multiple_error(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  // Insert first instance directly via repository
  db.upsert_app_instance("client-one", "secret-one", "scope-one", "setup")
    .await?;
  // Insert second instance with a different client_id
  db.upsert_app_instance("client-two", "secret-two", "scope-two", "setup")
    .await?;

  // Now get_instance should detect two rows and return error
  let svc = make_service(db);
  let result = svc.get_instance().await;
  let err = result.unwrap_err();
  // Propagates as Db(DbError::MultipleAppInstance)
  assert_eq!("db_error-multiple_app_instance", err.code());
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
  db.upsert_app_instance(TEST_CLIENT_ID, secret, "test-scope", "ready")
    .await?;

  let row = db.get_app_instance().await?.expect("row should exist");
  // The repository decrypts transparently: client_secret holds plaintext
  assert_eq!(secret, row.client_secret);
  assert_eq!(TEST_CLIENT_ID, row.client_id);
  assert_eq!("test-scope", row.scope);
  assert_eq!("ready", row.app_status);
  Ok(())
}

// =========================================================================
// repository: update_app_instance_status on non-existent row returns error
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_status_nonexistent_client_id_returns_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let result = db
    .update_app_instance_status("nonexistent-client", "ready")
    .await;
  let err = result.unwrap_err();
  assert_eq!("db_error-item_not_found", err.code());
  Ok(())
}

// =========================================================================
// repository: delete_app_instance removes the row
// =========================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_app_instance(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  db.upsert_app_instance(TEST_CLIENT_ID, TEST_CLIENT_SECRET, "scope", "ready")
    .await?;
  db.delete_app_instance(TEST_CLIENT_ID).await?;
  let row = db.get_app_instance().await?;
  assert!(row.is_none());
  Ok(())
}
