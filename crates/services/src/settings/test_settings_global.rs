use crate::{
  settings::{DbSetting, SettingsRepository},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

/// Verify that settings are global (no tenant_id column) and readable regardless of
/// which tenant context is active. Settings are NOT isolated per tenant — this is by
/// design since settings control app-wide behavior (log level, exec variant, etc.).
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_settings_are_global_not_tenant_scoped(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let setting = DbSetting {
    key: "global_test_setting".to_string(),
    value: "global_value".to_string(),
    value_type: "string".to_string(),
    created_at: DateTime::<Utc>::UNIX_EPOCH,
    updated_at: DateTime::<Utc>::UNIX_EPOCH,
  };
  ctx.service.upsert_setting(&setting).await?;

  // Settings are accessible without any tenant context — no tenant_id parameter
  let fetched = ctx.service.get_setting("global_test_setting").await?;
  assert!(fetched.is_some());
  assert_eq!("global_value", fetched.unwrap().value);

  let all = ctx.service.list_settings().await?;
  assert!(all.iter().any(|s| s.key == "global_test_setting"));

  Ok(())
}
