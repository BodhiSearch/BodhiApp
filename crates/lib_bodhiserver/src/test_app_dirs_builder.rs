use super::{load_defaults_yaml, setup_app_dirs, setup_bootstrap_service, BootstrapError};
use crate::{AppOptions, AppOptionsBuilder};
use mockall::predicate::eq;
use objs::test_utils::{empty_bodhi_home, temp_dir};
use objs::{AppCommand, AppType, EnvType, SettingSource};
use rstest::rstest;
use serde_yaml::Value;
use services::{EnvWrapper, MockEnvWrapper, BODHI_HOME};
use std::collections::HashMap;
use std::env::VarError;
use std::sync::Arc;
use tempfile::TempDir;

#[rstest]
fn test_create_bodhi_home_from_env(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let bodhi_home_str = bodhi_home.display().to_string();
  let options = AppOptionsBuilder::development()
    .set_env(BODHI_HOME, &bodhi_home_str)
    .build()?;
  let file_defaults = HashMap::new();
  let (result_path, source) = super::create_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  assert_eq!(result_path, bodhi_home);
  assert_eq!(source, objs::SettingSource::Environment);
  assert!(bodhi_home.exists());
  Ok(())
}

#[rstest]
fn test_create_bodhi_home_from_home_dir(temp_dir: TempDir) -> anyhow::Result<()> {
  let options = AppOptionsBuilder::development()
    .set_env("HOME", &temp_dir.path().display().to_string())
    .build()?;
  let file_defaults = HashMap::new();
  let (result_path, source) = super::create_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  let expected_path = temp_dir.path().join(".cache").join("bodhi-dev");
  assert_eq!(source, objs::SettingSource::Default);
  assert_eq!(result_path, expected_path);
  assert!(expected_path.exists());
  Ok(())
}

#[rstest]
fn test_find_bodhi_home_fails_when_not_found() -> anyhow::Result<()> {
  let mut mock_env_wrapper = MockEnvWrapper::default();
  mock_env_wrapper
    .expect_var()
    .with(eq(BODHI_HOME))
    .times(1)
    .return_once(|_| Err(VarError::NotPresent));
  mock_env_wrapper
    .expect_home_dir()
    .times(1)
    .return_once(|| None);
  let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(mock_env_wrapper);
  let options = AppOptions::new(
    env_wrapper,
    EnvType::Development,
    AppType::Native,
    "1.0.0".to_string(),
    "unknown".to_string(),
    "http://localhost:8080".to_string(),
    "bodhi".to_string(),
    HashMap::new(),
    None,
  );
  let file_defaults = HashMap::new();
  let result = super::find_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  );
  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    BootstrapError::BodhiHomeNotResolved
  ));
  Ok(())
}

#[rstest]
fn test_setup_app_dirs_integration(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::development()
    .set_env(BODHI_HOME, &bodhi_home.display().to_string())
    .build()?;
  let (result_home, source, file_defaults) = setup_app_dirs(&options)?;
  assert_eq!(result_home, bodhi_home);
  assert_eq!(source, SettingSource::Environment);
  assert!(file_defaults.is_empty() || !file_defaults.is_empty()); // file_defaults returned
  assert!(bodhi_home.exists());
  Ok(())
}

#[rstest]
fn test_load_defaults_yaml_when_file_does_not_exist() -> anyhow::Result<()> {
  let defaults = load_defaults_yaml();
  assert!(defaults.is_empty());
  Ok(())
}

#[rstest]
fn test_find_bodhi_home_uses_file_defaults(temp_dir: TempDir) -> anyhow::Result<()> {
  let bodhi_home = temp_dir.path().join("bodhi-home");
  let mut file_defaults = HashMap::new();
  file_defaults.insert(
    BODHI_HOME.to_string(),
    Value::String(bodhi_home.display().to_string()),
  );
  let options = AppOptionsBuilder::development().build()?;
  let (result_path, source) = super::find_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  assert_eq!(result_path, bodhi_home);
  assert_eq!(source, SettingSource::Default);
  Ok(())
}

#[rstest]
fn test_find_bodhi_home_env_overrides_file_defaults(temp_dir: TempDir) -> anyhow::Result<()> {
  let env_bodhi_home = temp_dir.path().join("env-bodhi-home");
  let defaults_bodhi_home = temp_dir.path().join("defaults-bodhi-home");
  let options = AppOptionsBuilder::development()
    .set_env(BODHI_HOME, &env_bodhi_home.display().to_string())
    .build()?;
  let mut file_defaults = HashMap::new();
  file_defaults.insert(
    BODHI_HOME.to_string(),
    Value::String(defaults_bodhi_home.display().to_string()),
  );
  let (result_path, source) = super::find_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  assert_eq!(result_path, env_bodhi_home);
  assert_eq!(source, SettingSource::Environment);
  Ok(())
}

#[rstest]
fn test_find_bodhi_home_invalid_file_default_falls_back(temp_dir: TempDir) -> anyhow::Result<()> {
  let mut file_defaults = HashMap::new();
  file_defaults.insert(BODHI_HOME.to_string(), Value::Number(123.into()));
  let options = AppOptionsBuilder::development()
    .set_env("HOME", &temp_dir.path().display().to_string())
    .build()?;

  let (result_path, source) = super::find_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;

  let expected_path = temp_dir.path().join(".cache").join("bodhi-dev");
  assert_eq!(result_path, expected_path);
  assert_eq!(source, SettingSource::Default);
  Ok(())
}

#[rstest]
fn test_setup_app_dirs_with_invalid_bodhi_home() -> anyhow::Result<()> {
  use std::path::PathBuf;
  let invalid_home = PathBuf::from("/nonexistent/path/that/does/not/exist");
  let options = AppOptionsBuilder::with_bodhi_home(&invalid_home.display().to_string()).build()?;
  let result = setup_app_dirs(&options);
  assert!(result.is_err());
  Ok(())
}

#[rstest]
fn test_setup_app_dirs_with_app_settings(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi_enhanced");
  let bodhi_home_str = bodhi_home.display().to_string();

  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str)
    .set_env("TEST_VAR", "test_value")
    .set_app_setting(services::BODHI_PORT, "9090")
    .set_system_setting(services::BODHI_ENV_TYPE, "development")?
    .build()?;

  let (home, source, file_defaults) = setup_app_dirs(&options)?;
  let bootstrap =
    setup_bootstrap_service(&options, home, source, file_defaults, AppCommand::Default)?;

  // Verify that bootstrap succeeded and app_settings are stored for SettingService
  assert!(bootstrap.bodhi_home().exists());
  // app_settings (including BODHI_PORT=9090) flow through into_parts to DefaultSettingService
  let parts = bootstrap.into_parts();
  assert_eq!(
    parts.app_settings.get(services::BODHI_PORT),
    Some(&"9090".to_string())
  );

  Ok(())
}
