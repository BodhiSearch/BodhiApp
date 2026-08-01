use super::AppServiceBuilder;
use crate::{setup_app_dirs, setup_bootstrap_service, AppOptionsBuilder, BootstrapError};
use rstest::rstest;
use services::test_utils::empty_bodhi_home;
use services::AppCommand;
use services::{test_utils::FrozenTimeService, AppService, MockCacheService};
use std::sync::Arc;
use tempfile::TempDir;

#[rstest]
#[case(&BootstrapError::ServiceAlreadySet("test_service".to_string()), "test_service")]
fn test_error_messages_objs(#[case] error: &BootstrapError, #[case] expected: &str) {
  assert_eq!(expected, error.to_string());
}

#[rstest]
#[tokio::test]
async fn test_app_service_builder_new(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");

  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let (home, source, file_defaults) = setup_app_dirs(&options)?;
  let bootstrap =
    setup_bootstrap_service(&options, home, source, file_defaults, AppCommand::Default)?;
  let expected_home = bootstrap.bodhi_home();
  let parts = bootstrap.into_parts();
  let builder = AppServiceBuilder::new(parts);
  assert_eq!(expected_home, bodhi_home);
  drop(builder);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_app_service_builder_with_custom_services(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let (home, source, file_defaults) = setup_app_dirs(&options)?;
  let bootstrap =
    setup_bootstrap_service(&options, home, source, file_defaults, AppCommand::Default)?;

  let time_service = Arc::new(FrozenTimeService::default());

  let app_service = AppServiceBuilder::new(bootstrap.into_parts())
    .time_service(time_service.clone())?
    .build()
    .await?;

  assert_eq!(app_service.setting_service().bodhi_home().await, bodhi_home);

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_app_service_builder_with_multiple_services(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let bodhi_home_str = bodhi_home.display().to_string();

  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
  let (home, source, file_defaults) = setup_app_dirs(&options)?;
  let bootstrap =
    setup_bootstrap_service(&options, home, source, file_defaults, AppCommand::Default)?;

  let app_service = AppServiceBuilder::new(bootstrap.into_parts())
    .time_service(Arc::new(FrozenTimeService::default()))?
    .cache_service(Arc::new(MockCacheService::new()))?
    .build()
    .await?;

  assert_eq!(app_service.setting_service().bodhi_home().await, bodhi_home);

  Ok(())
}

#[rstest]
fn test_service_already_set_errors(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let (home, source, file_defaults) = setup_app_dirs(&options)?;
  let bootstrap =
    setup_bootstrap_service(&options, home, source, file_defaults, AppCommand::Default)?;
  let time_service = Arc::new(FrozenTimeService::default());

  let builder =
    AppServiceBuilder::new(bootstrap.into_parts()).time_service(time_service.clone())?;

  let result = builder.time_service(time_service);

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    BootstrapError::ServiceAlreadySet(service) if service == *"time_service"));

  Ok(())
}

#[rstest]
#[case("your-strong-encryption-key-here")] // current .env.example placeholder
#[case("your-encryption-key-here")] // legacy placeholder, 24 chars — passes the length floor
fn test_validate_encryption_key_rejects_placeholders(#[case] key: &str) {
  let err = super::validate_encryption_key(key).unwrap_err();
  assert!(matches!(err, BootstrapError::PlaceholderValue(v) if v == key));
}

#[rstest]
#[case("dummy-key")]
#[case("testkey")]
#[case("local-dev-key")]
#[case("test-encryption-key")] // 19 chars — one under the floor
fn test_validate_encryption_key_rejects_short_keys(#[case] key: &str) {
  let err = super::validate_encryption_key(key).unwrap_err();
  assert!(matches!(
    err,
    BootstrapError::WeakEncryptionKey { min: 20, actual } if actual == key.len()
  ));
}

#[rstest]
#[case("bodhi-integration-test-enc-key")] // the fixture key
#[case("NORMAL6puzzle8rusting7dada")] // generated passphrase, 26 chars
#[case("cmVwbGFjZS1tZS13aXRoLW9wZW5zc2wtcmFuZC1iYXNlNjQ=")] // openssl rand -base64 32 shape
fn test_validate_encryption_key_accepts_strong_keys(#[case] key: &str) -> anyhow::Result<()> {
  super::validate_encryption_key(key)?;
  Ok(())
}
