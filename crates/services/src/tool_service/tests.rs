use crate::db::{ApiKeyUpdate, AppToolsetConfigRow, MockTimeService, ToolsetRow};
use crate::exa_service::MockExaService;
use crate::test_utils::MockDbService;
use crate::tool_service::{DefaultToolService, ToolService, ToolsetError};
use anyhow_trace::anyhow_trace;
use chrono::{TimeZone, Utc};
use mockall::predicate::eq;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn test_toolset_row(id: &str, user_id: &str, name: &str) -> ToolsetRow {
  ToolsetRow {
    id: id.to_string(),
    user_id: user_id.to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    name: name.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: Some("encrypted".to_string()),
    salt: Some("salt".to_string()),
    nonce: Some("nonce".to_string()),
    created_at: 1700000000,
    updated_at: 1700000000,
  }
}

fn app_config_enabled() -> AppToolsetConfigRow {
  AppToolsetConfigRow {
    id: 1,
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    enabled: true,
    updated_by: "admin".to_string(),
    created_at: 1700000000,
    updated_at: 1700000000,
  }
}

// ============================================================================
// Static Method Tests
// ============================================================================

#[rstest]
fn test_list_all_tool_definitions() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let defs = service.list_all_tool_definitions();

  assert_eq!(1, defs.len());
  assert_eq!("builtin-exa-web-search", defs[0].function.name);
}

#[rstest]
fn test_list_types_returns_builtin_toolsets() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let types = service.list_types();

  assert_eq!(1, types.len());
  assert_eq!("4ff0e163-36fb-47d6-a5ef-26e396f067d6", types[0].scope_uuid);
  assert_eq!("scope_toolset-builtin-exa-web-search", types[0].scope);
  assert_eq!(4, types[0].tools.len());
}

#[rstest]
fn test_get_type_returns_toolset_definition() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let def = service.get_type("4ff0e163-36fb-47d6-a5ef-26e396f067d6");

  assert!(def.is_some());
  let def = def.unwrap();
  assert_eq!("Exa Web Search", def.name);
  assert_eq!(4, def.tools.len());
}

#[rstest]
fn test_get_type_returns_none_for_unknown() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let def = service.get_type("unknown");

  assert!(def.is_none());
}

#[rstest]
fn test_validate_type_success() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.validate_type("4ff0e163-36fb-47d6-a5ef-26e396f067d6");

  assert!(result.is_ok());
}

#[rstest]
fn test_validate_type_fails_for_unknown() {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.validate_type("unknown");

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::InvalidToolsetType(_)
  ));
}

// ============================================================================
// list_tools_for_user Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_tools_for_user_returns_tools_for_enabled_instances() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| {
      Ok(vec![
        test_toolset_row("id1", "user123", "my-exa-1"),
        ToolsetRow {
          enabled: false,
          ..test_toolset_row("id2", "user123", "my-exa-2")
        },
      ])
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let tools = service.list_tools_for_user("user123").await?;

  assert_eq!(4, tools.len());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_tools_for_user_returns_empty_when_no_instances() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| Ok(vec![]));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let tools = service.list_tools_for_user("user123").await?;

  assert!(tools.is_empty());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_tools_for_user_deduplicates_same_type() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| {
      Ok(vec![
        test_toolset_row("id1", "user123", "my-exa-1"),
        test_toolset_row("id2", "user123", "my-exa-2"),
      ])
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let tools = service.list_tools_for_user("user123").await?;

  assert_eq!(4, tools.len());
  Ok(())
}

// ============================================================================
// list_all_toolsets Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_all_toolsets_returns_toolsets_with_app_status() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolsets = service.list_all_toolsets().await?;

  assert_eq!(1, toolsets.len());
  assert!(toolsets[0].app_enabled);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_all_toolsets_shows_disabled_when_no_config() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolsets = service.list_all_toolsets().await?;

  assert_eq!(1, toolsets.len());
  assert!(!toolsets[0].app_enabled);
  Ok(())
}

// ============================================================================
// list Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_returns_user_toolsets() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| {
      Ok(vec![
        test_toolset_row("id1", "user123", "my-exa-1"),
        test_toolset_row("id2", "user123", "my-exa-2"),
      ])
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolsets = service.list("user123").await?;

  assert_eq!(2, toolsets.len());
  assert_eq!("my-exa-1", toolsets[0].name);
  assert_eq!("my-exa-2", toolsets[1].name);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_list_returns_empty_for_user_with_no_toolsets() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_list_toolsets()
    .with(eq("user123"))
    .returning(|_| Ok(vec![]));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolsets = service.list("user123").await?;

  assert!(toolsets.is_empty());
  Ok(())
}

// ============================================================================
// get Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_get_returns_owned_toolset() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa-1"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service.get("user123", "id1").await?;

  assert!(toolset.is_some());
  assert_eq!("my-exa-1", toolset.unwrap().name);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_get_returns_none_for_other_users_toolset() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service.get("user123", "id1").await?;

  assert!(toolset.is_none());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_get_returns_none_when_not_found() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id999"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service.get("user123", "id999").await?;

  assert!(toolset.is_none());
  Ok(())
}

// ============================================================================
// create Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_success() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000000, 0).unwrap());

  db.expect_get_toolset_by_name()
    .with(eq("user123"), eq("my-exa"))
    .returning(|_, _| Ok(None));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_create_toolset().returning(|row| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .create(
      "user123",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      "my-exa",
      Some("My Exa".to_string()),
      true,
      "test-api-key".to_string(),
    )
    .await?;

  assert_eq!("my-exa", toolset.name);
  assert!(toolset.enabled);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_fails_when_name_already_exists() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_get_toolset_by_name()
    .with(eq("user123"), eq("my-exa"))
    .returning(|_, _| Ok(Some(test_toolset_row("existing", "user123", "my-exa"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .create(
      "user123",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      "my-exa",
      None,
      true,
      "test-api-key".to_string(),
    )
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), ToolsetError::NameExists(_)));
  Ok(())
}

#[rstest]
#[case("", "empty")]
#[case("my_toolset", "special chars")]
#[tokio::test]
async fn test_create_fails_with_invalid_name(
  #[case] name: &str,
  #[case] _reason: &str,
) -> anyhow::Result<()> {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .create(
      "user123",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      name,
      None,
      true,
      "test-api-key".to_string(),
    )
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), ToolsetError::InvalidName(_)));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_fails_with_too_long_name() -> anyhow::Result<()> {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let long_name = "a".repeat(25);
  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .create(
      "user123",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      &long_name,
      None,
      true,
      "test-api-key".to_string(),
    )
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), ToolsetError::InvalidName(_)));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_fails_with_invalid_toolset_type() -> anyhow::Result<()> {
  let db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .create(
      "user123",
      "unknown-type",
      "my-exa",
      None,
      true,
      "test-api-key".to_string(),
    )
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::InvalidToolsetType(_)
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_fails_when_app_disabled() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .create(
      "user123",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      "my-exa",
      None,
      true,
      "test-api-key".to_string(),
    )
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetAppDisabled
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_create_same_name_different_user_succeeds() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000000, 0).unwrap());

  db.expect_get_toolset_by_name()
    .with(eq("user456"), eq("my-exa"))
    .returning(|_, _| Ok(None));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_create_toolset().returning(|row| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .create(
      "user456",
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
      "my-exa",
      None,
      true,
      "test-api-key".to_string(),
    )
    .await?;

  assert_eq!("my-exa", toolset.name);
  Ok(())
}

// ============================================================================
// update Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_success() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
  db.expect_get_toolset_by_name()
    .with(eq("user123"), eq("my-exa-updated"))
    .returning(|_, _| Ok(None));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_update_toolset()
    .returning(|row, _| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .update(
      "user123",
      "id1",
      "my-exa-updated",
      Some("Updated".to_string()),
      true,
      ApiKeyUpdate::Keep,
    )
    .await?;

  assert_eq!("my-exa-updated", toolset.name);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_fails_when_not_found() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id999"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .update("user123", "id999", "my-exa", None, true, ApiKeyUpdate::Keep)
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetNotFound(_)
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_fails_when_not_owned() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetNotFound(_)
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_fails_when_name_conflicts() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa-1"))));
  db.expect_get_toolset_by_name()
    .with(eq("user123"), eq("my-exa-2"))
    .returning(|_, _| Ok(Some(test_toolset_row("id2", "user123", "my-exa-2"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .update("user123", "id1", "my-exa-2", None, true, ApiKeyUpdate::Keep)
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), ToolsetError::NameExists(_)));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_same_name_different_case_succeeds() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "MyExa"))));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_update_toolset()
    .returning(|row, _| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .update("user123", "id1", "myexa", None, true, ApiKeyUpdate::Keep)
    .await?;

  assert_eq!("myexa", toolset.name);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_enable_fails_when_app_disabled() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset().with(eq("id1")).returning(|_| {
    Ok(Some(ToolsetRow {
      enabled: false,
      ..test_toolset_row("id1", "user123", "my-exa")
    }))
  });
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service
    .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetAppDisabled
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_with_api_key_set() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_update_toolset()
    .returning(|row, _| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .update(
      "user123",
      "id1",
      "my-exa",
      None,
      true,
      ApiKeyUpdate::Set(Some("new-key".to_string())),
    )
    .await?;

  assert_eq!("my-exa", toolset.name);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_update_with_api_key_keep() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let mut time = MockTimeService::new();

  time
    .expect_utc_now()
    .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));
  db.expect_encryption_key()
    .return_const(b"0123456789abcdef".to_vec());
  db.expect_update_toolset()
    .returning(|row, _| Ok(row.clone()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let toolset = service
    .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
    .await?;

  assert_eq!("my-exa", toolset.name);
  Ok(())
}

// ============================================================================
// delete Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_delete_success() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
  db.expect_delete_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.delete("user123", "id1").await;

  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_delete_fails_when_not_found() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id999"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.delete("user123", "id999").await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetNotFound(_)
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_delete_fails_when_not_owned() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.delete("user123", "id1").await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    ToolsetError::ToolsetNotFound(_)
  ));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_delete_succeeds_even_when_app_disabled() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
  db.expect_delete_toolset()
    .with(eq("id1"))
    .returning(|_| Ok(()));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let result = service.delete("user123", "id1").await;

  assert!(result.is_ok());
  Ok(())
}

// ============================================================================
// is_type_enabled Tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_type_enabled_true() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(Some(app_config_enabled())));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let enabled = service
    .is_type_enabled("4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(enabled);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_type_enabled_false_when_disabled() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| {
      Ok(Some(AppToolsetConfigRow {
        enabled: false,
        ..app_config_enabled()
      }))
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let enabled = service
    .is_type_enabled("4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(!enabled);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_type_enabled_false_when_no_config() -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_toolset_config_by_scope()
    .with(eq("scope_toolset-builtin-exa-web-search"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let enabled = service
    .is_type_enabled("4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(!enabled);
  Ok(())
}

// ============================================================================
// App-client toolset registration tests
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_true_when_registered(
) -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_client_toolset_config()
    .with(eq("external-app"))
    .returning(|_| {
      Ok(Some(crate::db::AppClientToolsetConfigRow {
        id: 1,
        app_client_id: "external-app".to_string(),
        config_version: Some("v1.0.0".to_string()),
        toolsets_json: r#"[{"scope_id":"4ff0e163-36fb-47d6-a5ef-26e396f067d6","toolset_scope":"scope_toolset-builtin-exa-web-search"}]"#.to_string(),
        resource_scope: "scope_resource-bodhi".to_string(),
        created_at: 0,
        updated_at: 0,
      }))
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let registered = service
    .is_app_client_registered_for_toolset("external-app", "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(registered);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_false_when_toolset_not_in_config(
) -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_client_toolset_config()
    .with(eq("external-app"))
    .returning(|_| {
      Ok(Some(crate::db::AppClientToolsetConfigRow {
        id: 1,
        app_client_id: "external-app".to_string(),
        config_version: Some("v1.0.0".to_string()),
        toolsets_json:
          r#"[{"scope_id":"other-uuid","toolset_scope":"scope_toolset-other-toolset"}]"#
            .to_string(),
        resource_scope: "scope_resource-bodhi".to_string(),
        created_at: 0,
        updated_at: 0,
      }))
    });

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let registered = service
    .is_app_client_registered_for_toolset("external-app", "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(!registered);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_false_when_no_config(
) -> anyhow::Result<()> {
  let mut db = MockDbService::new();
  let exa = MockExaService::new();
  let time = MockTimeService::new();

  db.expect_get_app_client_toolset_config()
    .with(eq("unknown-app"))
    .returning(|_| Ok(None));

  let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), false);
  let registered = service
    .is_app_client_registered_for_toolset("unknown-app", "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .await?;

  assert!(!registered);
  Ok(())
}
