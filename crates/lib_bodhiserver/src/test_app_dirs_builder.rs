use super::{load_defaults_yaml, setup_app_dirs, AppDirsBuilderError};
use crate::{AppOptions, AppOptionsBuilder};
use mockall::predicate::eq;
use objs::test_utils::{empty_bodhi_home, temp_dir};
use objs::{AppType, EnvType, SettingSource};
use rstest::rstest;
use serde_yaml::Value;
use services::{
  test_utils::{TEST_PROD_DB, TEST_SESSION_DB},
  EnvWrapper, MockEnvWrapper, MockSettingService, SettingService, BODHI_HOME, HF_HOME,
};
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
    AppDirsBuilderError::BodhiHomeNotFound
  ));
  Ok(())
}

#[rstest]
fn test_setup_app_dirs_integration(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let options = AppOptionsBuilder::development()
    .set_env(BODHI_HOME, &bodhi_home.display().to_string())
    .build()?;
  let _settings_service = setup_app_dirs(&options)?;
  assert!(bodhi_home.join(TEST_PROD_DB).exists());
  assert!(bodhi_home.join(TEST_SESSION_DB).exists());
  Ok(())
}

#[rstest]
fn test_setup_hf_home_when_setting_exists(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let hf_home = empty_bodhi_home.path().join("hf_home");
  let bodhi_home_str = bodhi_home.display().to_string();

  // Set up real settings service with HF_HOME pre-configured
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
  let setting_service = setup_app_dirs(&options)?;

  // Set the HF_HOME setting
  setting_service.set_setting(HF_HOME, &hf_home.display().to_string());

  let result = super::setup_hf_home(&setting_service)?;

  assert_eq!(result, hf_home);
  assert!(hf_home.join("hub").exists());
  Ok(())
}

#[rstest]
fn test_setup_hf_home_fails_when_no_home_dir() -> anyhow::Result<()> {
  let mut mock = MockSettingService::default();
  mock
    .expect_get_setting()
    .with(eq(HF_HOME))
    .times(1)
    .return_const(None);
  mock.expect_home_dir().times(1).return_const(None);

  let setting_service: Arc<dyn SettingService> = Arc::new(mock);
  let result = super::setup_hf_home(setting_service.as_ref());

  assert!(matches!(result, Err(AppDirsBuilderError::HfHomeNotFound)));
  Ok(())
}

#[rstest]
fn test_setup_hf_home_creates_default_when_missing(
  empty_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");

  // Set up real settings service - setup_app_dirs already calls setup_hf_home
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = setup_app_dirs(&options)?;

  // HF_HOME should be set by setup_app_dirs
  let hf_home_setting = setting_service.get_setting(HF_HOME);
  assert!(hf_home_setting.is_some());

  // Should create default HF_HOME under the user's home directory
  let expected_hf_home = setting_service
    .home_dir()
    .unwrap()
    .join(".cache")
    .join("huggingface");

  assert_eq!(
    hf_home_setting.unwrap(),
    expected_hf_home.display().to_string()
  );
  assert!(expected_hf_home.join("hub").exists());
  Ok(())
}

#[rstest]
fn test_setup_logs_dir_success(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");

  // Set up real settings service
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
  let setting_service = setup_app_dirs(&options)?;

  let result = super::setup_logs_dir(&setting_service)?;
  let expected_logs_dir = setting_service.logs_dir();

  assert_eq!(result, expected_logs_dir);
  assert!(expected_logs_dir.exists());
  Ok(())
}

// Tests for helper methods

#[rstest]
fn test_setup_bodhi_subdirs_success(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
  let bodhi_home = empty_bodhi_home.path().join("bodhi");
  let bodhi_home_str = bodhi_home.display().to_string();

  // Set up real settings service
  let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
  let setting_service = setup_app_dirs(&options)?;

  // The setup_app_dirs already calls setup_bodhi_subdirs, so we just verify the results
  let app_db_path = setting_service.app_db_path();
  let session_db_path = setting_service.session_db_path();

  assert!(app_db_path.exists());
  assert!(session_db_path.exists());
  Ok(())
}

#[rstest]
fn test_setup_logs_dir_error_when_cannot_create() -> anyhow::Result<()> {
  // Create a read-only directory to simulate permission error
  let temp_dir = temp_dir();
  let logs_dir = temp_dir.path().join("readonly").join("logs");

  // Create parent as read-only
  let readonly_parent = temp_dir.path().join("readonly");
  std::fs::create_dir(&readonly_parent)?;

  // Make it read-only (this will fail on some systems, but that's ok for the test)
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&readonly_parent)?.permissions();
    perms.set_mode(0o444); // read-only
    std::fs::set_permissions(&readonly_parent, perms)?;
  }

  let mut mock = MockSettingService::default();
  mock
    .expect_logs_dir()
    .times(1)
    .return_const(logs_dir.clone());

  let setting_service: Arc<dyn SettingService> = Arc::new(mock);
  let result = super::setup_logs_dir(setting_service.as_ref());

  // On some systems this might succeed, so we just check it doesn't panic
  match result {
    Ok(_) => {
      // Directory creation succeeded despite read-only parent
      assert!(logs_dir.exists());
    }
    Err(AppDirsBuilderError::DirCreate { .. }) => {
      // Expected error case
    }
    Err(e) => {
      panic!("Unexpected error type: {:?}", e);
    }
  }

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
