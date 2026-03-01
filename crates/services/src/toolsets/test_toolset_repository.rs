use crate::{
  db::{encryption::encrypt_api_key, ApiKeyUpdate},
  test_utils::{sea_context, setup_env},
  toolsets::{ToolsetRepository, ToolsetRow},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_toolset(id: &str, user_id: &str, slug: &str, now: DateTime<Utc>) -> ToolsetRow {
  ToolsetRow {
    id: id.to_string(),
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
  let id = ulid::Ulid::new().to_string();
  let row = make_toolset(&id, "user-001", "my-toolset", ctx.now);

  let created = ctx.service.create_toolset(&row).await?;
  assert_eq!(row, created);

  let fetched = ctx.service.get_toolset(&id).await?;
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
    .create_toolset(&make_toolset(
      &ulid::Ulid::new().to_string(),
      user_id,
      "toolset-1",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_toolset(&make_toolset(
      &ulid::Ulid::new().to_string(),
      user_id,
      "toolset-2",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_toolset(&make_toolset(
      &ulid::Ulid::new().to_string(),
      "other-user",
      "toolset-3",
      ctx.now,
    ))
    .await?;

  let toolsets = ctx.service.list_toolsets(user_id).await?;
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
  let id = ulid::Ulid::new().to_string();
  ctx
    .service
    .create_toolset(&make_toolset(&id, "user-001", "to-delete", ctx.now))
    .await?;

  ctx.service.delete_toolset(&id).await?;

  let fetched = ctx.service.get_toolset(&id).await?;
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
  let id = ulid::Ulid::new().to_string();
  let api_key = "test-api-key-12345";

  let (encrypted, salt, nonce) =
    encrypt_api_key(&ctx.service.encryption_key, api_key).expect("encryption should work");

  let mut row = make_toolset(&id, "user-001", "encrypted-toolset", ctx.now);
  row.encrypted_api_key = Some(encrypted);
  row.salt = Some(salt);
  row.nonce = Some(nonce);

  ctx.service.create_toolset(&row).await?;

  let decrypted = ctx.service.get_toolset_api_key(&id).await?;
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
  let id = ulid::Ulid::new().to_string();
  let row = make_toolset(&id, "user-001", "original-slug", ctx.now);
  ctx.service.create_toolset(&row).await?;

  let mut updated_row = row.clone();
  updated_row.slug = "updated-slug".to_string();
  updated_row.description = Some("Updated description".to_string());
  updated_row.enabled = false;

  ctx
    .service
    .update_toolset(&updated_row, ApiKeyUpdate::Keep)
    .await?;

  let fetched = ctx.service.get_toolset(&id).await?.expect("should exist");
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
    .set_app_toolset_enabled("builtin-exa-search", true, "admin")
    .await?;

  assert_eq!("builtin-exa-search", config.toolset_type);
  assert!(config.enabled);
  assert_eq!("admin", config.updated_by);

  let fetched = ctx
    .service
    .get_app_toolset_config("builtin-exa-search")
    .await?;
  assert!(fetched.is_some());
  assert!(fetched.unwrap().enabled);

  ctx
    .service
    .set_app_toolset_enabled("builtin-exa-search", false, "admin2")
    .await?;

  let updated = ctx
    .service
    .get_app_toolset_config("builtin-exa-search")
    .await?
    .unwrap();
  assert!(!updated.enabled);
  assert_eq!("admin2", updated.updated_by);

  let all = ctx.service.list_app_toolset_configs().await?;
  assert_eq!(1, all.len());

  Ok(())
}
