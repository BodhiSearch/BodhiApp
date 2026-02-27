use crate::{
  db::{DbSetting, SettingsRepository},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_setting(key: &str, value: &str, value_type: &str) -> DbSetting {
  DbSetting {
    key: key.to_string(),
    value: value.to_string(),
    value_type: value_type.to_string(),
    created_at: DateTime::<Utc>::UNIX_EPOCH,
    updated_at: DateTime::<Utc>::UNIX_EPOCH,
  }
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_upsert_and_get_setting(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let setting = make_setting("test_key", "test_value", "string");

  let created = ctx.service.upsert_setting(&setting).await?;
  assert_eq!("test_key", created.key);
  assert_eq!("test_value", created.value);
  assert_eq!("string", created.value_type);
  assert_eq!(ctx.now, created.created_at);
  assert_eq!(ctx.now, created.updated_at);

  let fetched = ctx.service.get_setting("test_key").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("test_key", fetched.key);
  assert_eq!("test_value", fetched.value);
  assert_eq!(ctx.now, fetched.created_at);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_upsert_update_existing(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let setting = make_setting("update_key", "original_value", "string");

  let created = ctx.service.upsert_setting(&setting).await?;
  assert_eq!("original_value", created.value);
  assert_eq!(ctx.now, created.created_at);

  // Update with new value
  let updated_setting = make_setting("update_key", "new_value", "number");
  let updated = ctx.service.upsert_setting(&updated_setting).await?;
  assert_eq!("new_value", updated.value);
  assert_eq!("number", updated.value_type);
  // created_at should remain the same from initial insert
  assert_eq!(ctx.now, updated.created_at);
  assert_eq!(ctx.now, updated.updated_at);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_get_setting_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_setting("nonexistent_key").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_delete_setting(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let setting = make_setting("delete_key", "to_delete", "string");
  ctx.service.upsert_setting(&setting).await?;

  // Verify it exists
  let fetched = ctx.service.get_setting("delete_key").await?;
  assert!(fetched.is_some());

  // Delete it
  ctx.service.delete_setting("delete_key").await?;

  // Verify it's gone
  let fetched = ctx.service.get_setting("delete_key").await?;
  assert_eq!(None, fetched);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_sea_list_settings_sorted_by_key(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Insert in non-alphabetical order
  ctx
    .service
    .upsert_setting(&make_setting("charlie", "c_val", "string"))
    .await?;
  ctx
    .service
    .upsert_setting(&make_setting("alpha", "a_val", "string"))
    .await?;
  ctx
    .service
    .upsert_setting(&make_setting("bravo", "b_val", "number"))
    .await?;

  let settings = ctx.service.list_settings().await?;
  assert_eq!(3, settings.len());
  assert_eq!("alpha", settings[0].key);
  assert_eq!("a_val", settings[0].value);
  assert_eq!("bravo", settings[1].key);
  assert_eq!("b_val", settings[1].value);
  assert_eq!("charlie", settings[2].key);
  assert_eq!("c_val", settings[2].value);

  Ok(())
}
