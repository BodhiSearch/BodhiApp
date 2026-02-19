use super::{AppServiceBuilder, AppServiceBuilderError};
use crate::{setup_app_dirs, AppOptionsBuilder};
use objs::test_utils::empty_bodhi_home;
use rstest::rstest;
use services::{
  test_utils::FrozenTimeService, AppService, MockCacheService, MockSecretService, SettingService,
};
use std::sync::Arc;
use tempfile::TempDir;

#[rstest]
#[case(&AppServiceBuilderError::ServiceAlreadySet("test_service".to_string()), "test_service")]
fn test_error_messages_objs(#[case] error: &AppServiceBuilderError, #[case] expected: &str) {
  assert_eq!(expected, error.to_string());
}

#[rstest]
#[tokio::test]
async fn test_app_service_builder_new(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");

  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);

  let builder = AppServiceBuilder::new(setting_service);

  assert_eq!(builder.setting_service.bodhi_home(), bodhi_home);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_app_service_builder_with_custom_services(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);

  // Create mock services
  let mut mock_secret_service = MockSecretService::new();
  mock_secret_service
    .expect_set_secret_string()
    .returning(|_, _| Ok(()));
  mock_secret_service
    .expect_get_secret_string()
    .returning(|_| Ok(Some("test_value".to_string())));

  let time_service = Arc::new(FrozenTimeService::default());

  let app_service = AppServiceBuilder::new(setting_service.clone())
    .secret_service(Arc::new(mock_secret_service))?
    .time_service(time_service.clone())?
    .build()
    .await?;

  // Verify all services are properly initialized
  assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
  assert!(setting_service.app_db_path().exists());
  assert!(setting_service.session_db_path().exists());

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
  let setting_service = Arc::new(setup_app_dirs(&options)?);

  // Create mock secret service
  let mut mock_secret_service = MockSecretService::new();
  mock_secret_service
    .expect_set_secret_string()
    .returning(|_, _| Ok(()));
  mock_secret_service
    .expect_get_secret_string()
    .returning(|_| Ok(Some("test_value".to_string())));

  let app_service = AppServiceBuilder::new(setting_service.clone())
    .secret_service(Arc::new(mock_secret_service))?
    .cache_service(Arc::new(MockCacheService::new()))?
    .build()
    .await?;

  // Verify all services are properly initialized
  assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
  assert!(setting_service.app_db_path().exists());
  assert!(setting_service.session_db_path().exists());

  Ok(())
}

#[rstest]
fn test_service_already_set_errors() -> anyhow::Result<()> {
  let setting_service = Arc::new(services::test_utils::SettingServiceStub::default());
  let mock_secret_service = Arc::new(MockSecretService::new());

  let builder =
    AppServiceBuilder::new(setting_service).secret_service(mock_secret_service.clone())?;

  // This should return an error due to duplicate service setting
  let result = builder.secret_service(mock_secret_service);

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    AppServiceBuilderError::ServiceAlreadySet(service) if service == *"secret_service"));

  Ok(())
}

#[rstest]
fn test_setup_app_dirs_with_app_settings(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  use services::BODHI_PORT;

  let bodhi_home = empty_bodhi_home.path().join("bodhi_enhanced");
  let bodhi_home_str = bodhi_home.display().to_string();

  // Create options with app settings
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str)
    .set_env("TEST_VAR", "test_value")
    .set_app_setting(BODHI_PORT, "9090")
    .set_system_setting(services::BODHI_ENV_TYPE, "development")?
    .build()?;

  let setting_service = setup_app_dirs(&options)?;

  // Verify configuration was applied
  assert_eq!(setting_service.get_setting(BODHI_PORT).unwrap(), "9090");

  // Verify the service is properly initialized
  assert!(!setting_service.is_production());
  assert!(setting_service.bodhi_home().exists());

  Ok(())
}
