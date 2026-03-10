use crate::{
  db::encryption::encrypt_api_key,
  new_ulid,
  test_utils::{sea_context, setup_env, TEST_TENANT_ID},
  toolsets::{ToolsetEntity, ToolsetRepository},
  RawApiKeyUpdate,
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_toolset(id: &str, user_id: &str, slug: &str, now: DateTime<Utc>) -> ToolsetEntity {
  ToolsetEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: user_id.to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    slug: slug.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: None,
    salt: None,
    nonce: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_create_and_get_toolset(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let row = make_toolset(&id, "user-001", "my-toolset", ctx.now);

  let created = ctx.service.create_toolset(TEST_TENANT_ID, &row).await?;
  assert_eq!(row, created);

  let fetched = ctx.service.get_toolset(TEST_TENANT_ID, &id).await?;
  assert!(fetched.is_some());
  assert_eq!(row, fetched.unwrap());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_list_toolsets(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = "user-001";

  ctx
    .service
    .create_toolset(
      TEST_TENANT_ID,
      &make_toolset(&new_ulid(), user_id, "toolset-1", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_toolset(
      TEST_TENANT_ID,
      &make_toolset(&new_ulid(), user_id, "toolset-2", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_toolset(
      TEST_TENANT_ID,
      &make_toolset(&new_ulid(), "other-user", "toolset-3", ctx.now),
    )
    .await?;

  let toolsets = ctx.service.list_toolsets(TEST_TENANT_ID, user_id).await?;
  assert_eq!(2, toolsets.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_delete_toolset(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  ctx
    .service
    .create_toolset(
      TEST_TENANT_ID,
      &make_toolset(&id, "user-001", "to-delete", ctx.now),
    )
    .await?;

  ctx.service.delete_toolset(TEST_TENANT_ID, &id).await?;

  let fetched = ctx.service.get_toolset(TEST_TENANT_ID, &id).await?;
  assert!(fetched.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_toolset_encrypted_api_key(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let api_key = "test-api-key-12345";

  let (encrypted, salt, nonce) =
    encrypt_api_key(&ctx.service.encryption_key, api_key).expect("encryption should work");

  let mut row = make_toolset(&id, "user-001", "encrypted-toolset", ctx.now);
  row.encrypted_api_key = Some(encrypted);
  row.salt = Some(salt);
  row.nonce = Some(nonce);

  ctx.service.create_toolset(TEST_TENANT_ID, &row).await?;

  let decrypted = ctx.service.get_toolset_api_key(TEST_TENANT_ID, &id).await?;
  assert_eq!(Some(api_key.to_string()), decrypted);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_toolset(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let row = make_toolset(&id, "user-001", "original-slug", ctx.now);
  ctx.service.create_toolset(TEST_TENANT_ID, &row).await?;

  let mut updated_row = row.clone();
  updated_row.slug = "updated-slug".to_string();
  updated_row.description = Some("Updated description".to_string());
  updated_row.enabled = false;

  ctx
    .service
    .update_toolset(TEST_TENANT_ID, &updated_row, RawApiKeyUpdate::Keep)
    .await?;

  let fetched = ctx
    .service
    .get_toolset(TEST_TENANT_ID, &id)
    .await?
    .expect("should exist");
  assert_eq!("updated-slug", fetched.slug);
  assert_eq!(Some("Updated description".to_string()), fetched.description);
  assert!(!fetched.enabled);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_app_toolset_config_crud(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let config = ctx
    .service
    .set_app_toolset_enabled(TEST_TENANT_ID, "builtin-exa-search", true, "admin")
    .await?;

  assert_eq!("builtin-exa-search", config.toolset_type);
  assert!(config.enabled);
  assert_eq!("admin", config.updated_by);

  let fetched = ctx
    .service
    .get_app_toolset_config(TEST_TENANT_ID, "builtin-exa-search")
    .await?;
  assert!(fetched.is_some());
  assert!(fetched.unwrap().enabled);

  ctx
    .service
    .set_app_toolset_enabled(TEST_TENANT_ID, "builtin-exa-search", false, "admin2")
    .await?;

  let updated = ctx
    .service
    .get_app_toolset_config(TEST_TENANT_ID, "builtin-exa-search")
    .await?
    .unwrap();
  assert!(!updated.enabled);
  assert_eq!("admin2", updated.updated_by);

  let all = ctx.service.list_app_toolset_configs(TEST_TENANT_ID).await?;
  assert_eq!(1, all.len());

  Ok(())
}

// =========================================================================
// get_toolset_by_slug
// =========================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_toolset_by_slug(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let row = make_toolset(&id, "user-001", "my-unique-slug", ctx.now);
  ctx.service.create_toolset(TEST_TENANT_ID, &row).await?;

  // Find by slug
  let found = ctx
    .service
    .get_toolset_by_slug(TEST_TENANT_ID, "user-001", "my-unique-slug")
    .await?;
  assert!(found.is_some());
  assert_eq!(id, found.unwrap().id);

  // Non-existent slug returns None
  let missing = ctx
    .service
    .get_toolset_by_slug(TEST_TENANT_ID, "user-001", "no-such-slug")
    .await?;
  assert!(missing.is_none());

  // Different user returns None
  let wrong_user = ctx
    .service
    .get_toolset_by_slug(TEST_TENANT_ID, "other-user", "my-unique-slug")
    .await?;
  assert!(wrong_user.is_none());

  Ok(())
}

// =========================================================================
// list_toolsets_by_toolset_type
// =========================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_list_toolsets_by_toolset_type(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = "user-001";

  // Create two toolsets of same type and one of different type
  let mut ts1 = make_toolset(&new_ulid(), user_id, "exa-1", ctx.now);
  ts1.toolset_type = "builtin-exa-search".to_string();

  let mut ts2 = make_toolset(&new_ulid(), user_id, "exa-2", ctx.now);
  ts2.toolset_type = "builtin-exa-search".to_string();

  let mut ts3 = make_toolset(&new_ulid(), user_id, "brave-1", ctx.now);
  ts3.toolset_type = "builtin-brave-search".to_string();

  ctx.service.create_toolset(TEST_TENANT_ID, &ts1).await?;
  ctx.service.create_toolset(TEST_TENANT_ID, &ts2).await?;
  ctx.service.create_toolset(TEST_TENANT_ID, &ts3).await?;

  // Filter by exa-search type
  let exa = ctx
    .service
    .list_toolsets_by_toolset_type(TEST_TENANT_ID, user_id, "builtin-exa-search")
    .await?;
  assert_eq!(2, exa.len());

  // Filter by brave-search type
  let brave = ctx
    .service
    .list_toolsets_by_toolset_type(TEST_TENANT_ID, user_id, "builtin-brave-search")
    .await?;
  assert_eq!(1, brave.len());

  // Non-existent type returns empty
  let none = ctx
    .service
    .list_toolsets_by_toolset_type(TEST_TENANT_ID, user_id, "nonexistent")
    .await?;
  assert_eq!(0, none.len());

  Ok(())
}
