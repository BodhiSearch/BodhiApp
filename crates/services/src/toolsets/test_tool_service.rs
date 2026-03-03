use crate::db::{DbError, DbService, TimeService};
use crate::models::{ApiKey, ApiKeyUpdate};
use crate::test_utils::{test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID};
use crate::toolsets::{
  DefaultToolService, MockExaService, ToolService, ToolsetEntity, ToolsetRequest,
};
use anyhow_trace::anyhow_trace;
use chrono::{TimeZone, Utc};
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn default_time_service() -> Arc<dyn TimeService> {
  Arc::new(FrozenTimeService::default())
}

fn create_form(
  toolset_type: &str,
  slug: &str,
  description: Option<String>,
  enabled: bool,
  api_key: &str,
) -> ToolsetRequest {
  ToolsetRequest {
    toolset_type: Some(toolset_type.to_string()),
    slug: slug.to_string(),
    description,
    enabled,
    api_key: ApiKeyUpdate::Set(ApiKey::some(api_key.to_string()).unwrap()),
  }
}

fn update_form(
  slug: &str,
  description: Option<String>,
  enabled: bool,
  api_key: ApiKeyUpdate,
) -> ToolsetRequest {
  ToolsetRequest {
    toolset_type: None,
    slug: slug.to_string(),
    description,
    enabled,
    api_key,
  }
}

fn test_toolset_row(id: &str, user_id: &str, slug: &str) -> ToolsetEntity {
  let ts = Utc.timestamp_opt(1700000000, 0).unwrap();
  ToolsetEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: user_id.to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    slug: slug.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: Some("encrypted".to_string()),
    salt: Some("salt".to_string()),
    nonce: Some("nonce".to_string()),
    created_at: ts,
    updated_at: ts,
  }
}

async fn setup_app_toolset_enabled(db: &dyn DbService) {
  db.set_app_toolset_enabled(TEST_TENANT_ID, "builtin-exa-search", true, "admin")
    .await
    .unwrap();
}

/// Helper to create a toolset for test setup
async fn test_create_toolset(
  db: &dyn DbService,
  tenant_id: &str,
  row: &ToolsetEntity,
) -> Result<ToolsetEntity, DbError> {
  db.create_toolset(tenant_id, row).await
}

// ============================================================================
// Static Method Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_types_returns_builtin_toolsets(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let types = service.list_types();

  assert_eq!(1, types.len());
  assert_eq!("builtin-exa-search", types[0].toolset_type);
  assert_eq!(4, types[0].tools.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_type_returns_toolset_definition(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let def = service.get_type("builtin-exa-search");

  assert!(def.is_some());
  let def = def.unwrap();
  assert_eq!("Exa Web Search", def.name);
  assert_eq!(4, def.tools.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_type_returns_none_for_unknown(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let def = service.get_type("unknown");

  assert!(def.is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_validate_type_success(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.validate_type("builtin-exa-search");

  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_validate_type_fails_for_unknown(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.validate_type("unknown");

  assert_eq!(
    "toolset_error-invalid_toolset_type",
    result.unwrap_err().code()
  );
  Ok(())
}

// ============================================================================
// list_tools_for_user Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_tools_for_user_returns_tools_for_enabled_instances(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa-1"),
  )
  .await?;
  test_create_toolset(
    db.as_ref(),
    "",
    &ToolsetEntity {
      enabled: false,
      ..test_toolset_row("id2", "user123", "my-exa-2")
    },
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let tools = service.list_tools_for_user("", "user123").await?;

  assert_eq!(4, tools.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_tools_for_user_returns_empty_when_no_instances(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let tools = service.list_tools_for_user("", "user123").await?;

  assert!(tools.is_empty());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_tools_for_user_deduplicates_same_type(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa-1"),
  )
  .await?;
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id2", "user123", "my-exa-2"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let tools = service.list_tools_for_user("", "user123").await?;

  assert_eq!(4, tools.len());
  Ok(())
}

// ============================================================================
// list_all_toolsets Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_all_toolsets_returns_toolsets(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolsets = service.list_all_toolsets().await?;

  assert_eq!(1, toolsets.len());
  assert_eq!("builtin-exa-search", toolsets[0].toolset_type);
  assert_eq!("Exa Web Search", toolsets[0].name);
  assert_eq!(4, toolsets[0].tools.len());
  Ok(())
}

// ============================================================================
// list Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_returns_user_toolsets(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa-1"),
  )
  .await?;
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id2", "user123", "my-exa-2"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolsets = service.list("", "user123").await?;

  assert_eq!(2, toolsets.len());
  assert_eq!("my-exa-1", toolsets[0].slug);
  assert_eq!("my-exa-2", toolsets[1].slug);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_returns_empty_for_user_with_no_toolsets(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolsets = service.list("", "user123").await?;

  assert!(toolsets.is_empty());
  Ok(())
}

// ============================================================================
// get Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_returns_owned_toolset(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa-1"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolset = service.get("", "user123", "id1").await?;

  assert!(toolset.is_some());
  assert_eq!("my-exa-1", toolset.unwrap().slug);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_returns_none_for_other_users_toolset(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user999", "other-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolset = service.get("", "user123", "id1").await?;

  assert!(toolset.is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_returns_none_when_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let toolset = service.get("", "user123", "id999").await?;

  assert!(toolset.is_none());
  Ok(())
}

// ============================================================================
// create Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_success(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = create_form(
    "builtin-exa-search",
    "my-exa",
    Some("My Exa".to_string()),
    true,
    "test-api-key",
  );
  let toolset = service.create(TEST_TENANT_ID, "user123", form).await?;

  assert_eq!("my-exa", toolset.slug);
  assert!(toolset.enabled);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_fails_when_name_already_exists(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("existing", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = create_form("builtin-exa-search", "my-exa", None, true, "test-api-key");
  let result = service.create(TEST_TENANT_ID, "user123", form).await;

  assert_eq!("toolset_error-slug_exists", result.unwrap_err().code());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_fails_with_invalid_toolset_type(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();

  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = create_form("unknown-type", "my-exa", None, true, "test-api-key");
  let result = service.create("", "user123", form).await;

  assert_eq!(
    "toolset_error-invalid_toolset_type",
    result.unwrap_err().code()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_same_name_different_user_succeeds(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  // user123 has "my-exa", user456 should be able to create the same slug
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = create_form("builtin-exa-search", "my-exa", None, true, "test-api-key");
  let toolset = service.create(TEST_TENANT_ID, "user456", form).await?;

  assert_eq!("my-exa", toolset.slug);
  Ok(())
}

// ============================================================================
// update Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_success(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form(
    "my-exa-updated",
    Some("Updated".to_string()),
    true,
    ApiKeyUpdate::Keep,
  );
  let toolset = service
    .update(TEST_TENANT_ID, "user123", "id1", form)
    .await?;

  assert_eq!("my-exa-updated", toolset.slug);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_fails_when_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form("my-exa", None, true, ApiKeyUpdate::Keep);
  let result = service.update("", "user123", "id999", form).await;

  assert_eq!(
    "toolset_error-toolset_not_found",
    result.unwrap_err().code()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_fails_when_not_owned(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user999", "other-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form("my-exa", None, true, ApiKeyUpdate::Keep);
  let result = service.update("", "user123", "id1", form).await;

  assert_eq!(
    "toolset_error-toolset_not_found",
    result.unwrap_err().code()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_fails_when_name_conflicts(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "my-exa-1"),
  )
  .await?;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id2", "user123", "my-exa-2"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form("my-exa-2", None, true, ApiKeyUpdate::Keep);
  let result = service.update(TEST_TENANT_ID, "user123", "id1", form).await;

  assert_eq!("toolset_error-slug_exists", result.unwrap_err().code());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_same_name_different_case_succeeds(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "MyExa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form("myexa", None, true, ApiKeyUpdate::Keep);
  let toolset = service
    .update(TEST_TENANT_ID, "user123", "id1", form)
    .await?;

  assert_eq!("myexa", toolset.slug);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_with_api_key_set(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form(
    "my-exa",
    None,
    true,
    ApiKeyUpdate::Set(ApiKey::some("new-key".to_string()).unwrap()),
  );
  let toolset = service
    .update(TEST_TENANT_ID, "user123", "id1", form)
    .await?;

  assert_eq!("my-exa", toolset.slug);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_with_api_key_keep(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_app_toolset_enabled(db.as_ref()).await;
  test_create_toolset(
    db.as_ref(),
    TEST_TENANT_ID,
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let form = update_form("my-exa", None, true, ApiKeyUpdate::Keep);
  let toolset = service
    .update(TEST_TENANT_ID, "user123", "id1", form)
    .await?;

  assert_eq!("my-exa", toolset.slug);
  Ok(())
}

// ============================================================================
// delete Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_success(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.delete("", "user123", "id1").await;

  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_fails_when_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.delete("", "user123", "id999").await;

  assert_eq!(
    "toolset_error-toolset_not_found",
    result.unwrap_err().code()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_fails_when_not_owned(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user999", "other-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.delete("", "user123", "id1").await;

  assert_eq!(
    "toolset_error-toolset_not_found",
    result.unwrap_err().code()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_succeeds_even_when_app_disabled(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  test_create_toolset(
    db.as_ref(),
    "",
    &test_toolset_row("id1", "user123", "my-exa"),
  )
  .await?;

  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let result = service.delete("", "user123", "id1").await;

  assert!(result.is_ok());
  Ok(())
}

// ============================================================================
// is_type_enabled Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_is_type_enabled_defaults_false_when_no_config(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let exa = MockExaService::new();
  let service = DefaultToolService::new(db, Arc::new(exa), default_time_service());
  let enabled = service.is_type_enabled("", "builtin-exa-search").await?;

  assert!(!enabled);
  Ok(())
}
