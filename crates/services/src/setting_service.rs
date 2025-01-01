use crate::{asref_impl, EnvWrapper};
use objs::{impl_error_from, AppError, ErrorType, IoError, SerdeYamlError};
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::fs;
use std::path::Path;
use std::{path::PathBuf, sync::Arc, sync::RwLock};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  SerdeYaml(#[from] SerdeYamlError),
  #[error("lock_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  LockError(String),
}

impl_error_from!(::std::io::Error, SettingServiceError::Io, ::objs::IoError);
impl_error_from!(
  ::serde_yaml::Error,
  SettingServiceError::SerdeYaml,
  ::objs::SerdeYamlError
);

type Result<T> = std::result::Result<T, SettingServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SettingService: std::fmt::Debug + Send + Sync {
  fn load(&self, path: &Path);

  fn load_default_env(&self, bodhi_home: &Path);

  fn home_dir(&self) -> Option<PathBuf>;

  fn get_env(&self, key: &str) -> Option<String>;

  fn get_setting(&self, key: &str) -> Option<String>;

  fn get_setting_value(&self, key: &str) -> Option<Value>;

  fn get_setting_or_default(&self, key: &str, default: &str) -> String;

  fn set_setting(&self, key: &str, value: &str) -> Result<()>;

  fn set_setting_value(&self, key: &str, value: &Value) -> Result<()>;

  fn delete_setting(&self, key: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  path: PathBuf,
  settings_lock: RwLock<()>,
}

impl DefaultSettingService {
  pub fn new(env_wrapper: Arc<dyn EnvWrapper>, path: PathBuf) -> Self {
    Self {
      env_wrapper,
      path,
      settings_lock: RwLock::new(()),
    }
  }

  fn read_settings(&self) -> Result<serde_yaml::Mapping> {
    let _guard = self
      .settings_lock
      .read()
      .map_err(|e| SettingServiceError::LockError(e.to_string()))?;

    if !self.path.exists() {
      return Ok(serde_yaml::Mapping::new());
    }
    let contents = fs::read_to_string(&self.path)?;
    Ok(serde_yaml::from_str(&contents)?)
  }

  fn write_settings(&self, settings: &serde_yaml::Mapping) -> Result<()> {
    let _guard = self
      .settings_lock
      .write()
      .map_err(|e| SettingServiceError::LockError(e.to_string()))?;

    let contents = serde_yaml::to_string(settings)?;
    fs::write(&self.path, contents)?;
    Ok(())
  }
}

impl SettingService for DefaultSettingService {
  fn load(&self, path: &Path) {
    self.env_wrapper.load(path);
  }

  fn load_default_env(&self, bodhi_home: &Path) {
    let env_file = bodhi_home.join(".env");
    if env_file.exists() {
      self.load(&env_file);
    }
  }

  fn home_dir(&self) -> Option<PathBuf> {
    self.env_wrapper.home_dir()
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.env_wrapper.var(key).ok()
  }

  fn get_setting(&self, key: &str) -> Option<String> {
    self
      .get_setting_value(key)
      .and_then(|value| value.as_str().map(ToOwned::to_owned))
  }

  fn get_setting_or_default(&self, key: &str, default: &str) -> String {
    self.get_setting(key).unwrap_or_else(|| default.to_string())
  }

  fn set_setting(&self, key: &str, value: &str) -> Result<()> {
    self.set_setting_value(key, &Value::String(value.to_owned()))
  }

  fn delete_setting(&self, key: &str) -> Result<()> {
    let mut settings = self.read_settings()?;
    settings.remove(key);
    self.write_settings(&settings)
  }

  fn get_setting_value(&self, key: &str) -> Option<Value> {
    if let Ok(value) = self.env_wrapper.var(key) {
      return Some(Value::String(value));
    }
    self
      .read_settings()
      .ok()
      .and_then(|settings| settings.get(key).cloned())
  }

  fn set_setting_value(&self, key: &str, value: &Value) -> Result<()> {
    let mut settings = self.read_settings()?;
    settings.insert(key.into(), value.clone());
    self.write_settings(&settings)
  }
}

pub fn set_setting<S, T>(slf: S, key: &str, value: T) -> Result<()>
where
  T: serde::Serialize,
  S: AsRef<dyn SettingService>,
{
  let value = serde_yaml::to_value(value)?;
  slf.as_ref().set_setting_value(key, &value)
}

pub fn get_setting<S, T>(slf: S, key: &str) -> Result<Option<T>>
where
  T: DeserializeOwned,
  S: AsRef<dyn SettingService>,
{
  match slf.as_ref().get_setting_value(key) {
    Some(value) => {
      let result = serde_yaml::from_value(value)?;
      Ok(Some(result))
    }
    None => Ok(None),
  }
}

asref_impl!(SettingService, DefaultSettingService);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::MockEnvWrapper;
  use mockall::predicate::eq;
  use objs::test_utils::temp_dir;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use tempfile::TempDir;

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct TestConfig {
    name: String,
    value: i32,
  }

  #[rstest]
  fn test_setting_service_read_from_file_if_env_not_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: file_value")?;
    let mut mock_env = MockEnvWrapper::default();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
    assert_eq!(
      Some("file_value".to_string()),
      service.get_setting("TEST_KEY"),
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_env_if_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: file_value")?;
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Ok("env_value".to_string()));
    let service = DefaultSettingService::new(Arc::new(mock_env), path);
    assert_eq!(
      Some("env_value".to_string()),
      service.get_setting("TEST_KEY")
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_complex_object_from_file_if_env_not_set(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(
      &path,
      r#"TEST_CONFIG:
  name: test
  value: 42
"#,
    )?;
    let mut mock_env = MockEnvWrapper::default();
    mock_env
      .expect_var()
      .with(eq("TEST_CONFIG"))
      .return_const(Err(std::env::VarError::NotPresent));
    let service = DefaultSettingService::new(Arc::new(mock_env), path);

    let test_config = TestConfig {
      name: "test".to_string(),
      value: 42,
    };

    let retrieved: Option<TestConfig> = get_setting(&service, "TEST_CONFIG")?;
    assert_eq!(Some(test_config), retrieved);
    Ok(())
  }

  #[rstest]
  fn test_setting_service_persistence(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    {
      let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
      service.set_setting("TEST_KEY", "test_value")?;
    }

    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .return_const(Err(std::env::VarError::NotPresent));

    {
      let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
      assert_eq!(
        Some("test_value".to_string()),
        service.get_setting("TEST_KEY"),
      );
    }
    let contents = std::fs::read_to_string(path)?;
    assert_eq!("TEST_KEY: test_value\n", contents);
    Ok(())
  }

  #[rstest]
  fn test_settings_service_delete(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let mut mock_env = MockEnvWrapper::new();

    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());

    service.set_setting("TEST_KEY", "test_value")?;
    assert_eq!(
      Some("test_value".to_string()),
      service.get_setting("TEST_KEY")
    );

    service.delete_setting("TEST_KEY")?;
    assert_eq!(None, service.get_setting("TEST_KEY"));
    let contents = std::fs::read_to_string(path)?;
    assert_eq!("{}\n", contents);

    // Delete non-existent key should still succeed
    service.delete_setting("NON_EXISTENT_KEY")?;
    Ok(())
  }

  #[rstest]
  fn test_settings_delete_complex_object(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let mut mock_env = MockEnvWrapper::new();

    mock_env
      .expect_var()
      .return_const(Err(std::env::VarError::NotPresent));

    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());

    let test_config = TestConfig {
      name: "test".to_string(),
      value: 42,
    };

    // Save complex object
    set_setting(&service, "TEST_CONFIG", &test_config)?;

    // Verify it exists
    let retrieved: Option<TestConfig> = get_setting(&service, "TEST_CONFIG").unwrap();
    assert_eq!(Some(test_config), retrieved);

    // Delete it
    service.delete_setting("TEST_CONFIG").unwrap();

    // Verify it's gone
    let deleted: Option<TestConfig> = get_setting(&service, "TEST_CONFIG").unwrap();
    assert_eq!(None, deleted);
    Ok(())
  }
}
