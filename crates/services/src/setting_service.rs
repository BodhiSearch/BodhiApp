use crate::{asref_impl, EnvWrapper, BODHI_HOME};
use objs::{
  impl_error_from, AppError, ErrorType, IoError, SerdeYamlError, SettingInfo, SettingMetadata,
  SettingSource,
};
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::fs;
use std::path::Path;
use std::{path::PathBuf, sync::Arc, sync::RwLock};

pub const HF_HOME: &str = "HF_HOME";
pub const BODHI_LOGS: &str = "BODHI_LOGS";
pub const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
pub const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";
pub const BODHI_SCHEME: &str = "BODHI_SCHEME";
pub const BODHI_HOST: &str = "BODHI_HOST";
pub const BODHI_PORT: &str = "BODHI_PORT";
pub const BODHI_EXEC_PATH: &str = "BODHI_EXEC_PATH";
pub const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
pub const BODHI_EXEC_VARIANT: &str = "BODHI_EXEC_VARIANT";

pub const DEFAULT_SCHEME: &str = "http";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 1135;
pub const DEFAULT_PORT_STR: &str = "1135";
pub const DEFAULT_LOG_LEVEL: &str = "warn";
pub const DEFAULT_LOG_STDOUT: bool = false;

pub const SETTINGS_YAML: &str = "settings.yaml";

pub const SETTING_VARS: &[&str] = &[
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  HF_HOME,
  BODHI_SCHEME,
  BODHI_HOST,
  BODHI_PORT,
  BODHI_EXEC_PATH,
  BODHI_EXEC_LOOKUP_PATH,
];

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

  fn list(&self) -> Vec<SettingInfo>;

  fn get_default_value(&self, key: &str) -> Value;

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata;

  fn get_env(&self, key: &str) -> Option<String>;

  fn get_setting(&self, key: &str) -> Option<String>;

  fn get_setting_value(&self, key: &str) -> Option<Value>;

  fn get_setting_value_with_source(&self, key: &str, default: Value) -> (Value, SettingSource);

  fn get_setting_or_default(&self, key: &str) -> String;

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

  pub fn read_settings(&self) -> Result<serde_yaml::Mapping> {
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
    self.get_setting_value(key).and_then(|value| match value {
      Value::String(s) => Some(s),
      Value::Number(n) => Some(n.to_string()),
      Value::Bool(b) => Some(b.to_string()),
      Value::Null => None,
      _ => None,
    })
  }

  fn get_setting_or_default(&self, key: &str) -> String {
    self.get_setting(key).unwrap_or_else(|| {
      let default_value = self.get_default_value(key);
      match default_value {
        Value::String(s) => s,
        Value::Bool(bool) => bool.to_string(),
        Value::Number(number) => number.to_string(),
        _ => "<unknown>".to_string(),
      }
    })
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

  fn get_setting_value_with_source(&self, key: &str, default: Value) -> (Value, SettingSource) {
    if let Ok(value) = self.env_wrapper.var(key) {
      return (Value::String(value), SettingSource::Environment);
    }
    self
      .read_settings()
      .ok()
      .and_then(|settings| settings.get(key).cloned())
      .map(|value| (value, SettingSource::SettingsFile))
      .unwrap_or((default, SettingSource::Default))
  }

  fn set_setting_value(&self, key: &str, value: &serde_yaml::Value) -> Result<()> {
    let mut settings = self.read_settings()?;
    settings.insert(key.into(), value.clone());
    self.write_settings(&settings)
  }

  fn list(&self) -> Vec<SettingInfo> {
    SETTING_VARS
      .iter()
      .map(|key| {
        let (current_value, source) =
          self.get_setting_value_with_source(key, self.get_default_value(key));
        let metadata = self.get_setting_metadata(key);
        let current_value = metadata.parse(current_value);

        SettingInfo {
          key: key.to_string(),
          current_value,
          default_value: self.get_default_value(key),
          source,
          metadata,
        }
      })
      .collect()
  }

  fn get_default_value(&self, key: &str) -> serde_yaml::Value {
    match key {
      BODHI_HOME => {
        let default_home = self
          .home_dir()
          .map(|home| home.join(".cache").join("bodhi").display().to_string());
        default_home
          .map(serde_yaml::Value::String)
          .unwrap_or(serde_yaml::Value::Null)
      }
      BODHI_LOGS => {
        let bodhi_logs = self.home_dir().map(|home| {
          home
            .join(".cache")
            .join("bodhi")
            .join("logs")
            .display()
            .to_string()
        });
        bodhi_logs
          .map(serde_yaml::Value::String)
          .unwrap_or(serde_yaml::Value::Null)
      }
      HF_HOME => {
        let default_hf_home = self.home_dir().map(|home| {
          home
            .join(".cache")
            .join("huggingface")
            .display()
            .to_string()
        });
        default_hf_home
          .map(serde_yaml::Value::String)
          .unwrap_or(serde_yaml::Value::Null)
      }
      BODHI_SCHEME => serde_yaml::Value::String(DEFAULT_SCHEME.to_string()),
      BODHI_HOST => serde_yaml::Value::String(DEFAULT_HOST.to_string()),
      BODHI_PORT => serde_yaml::Value::Number(DEFAULT_PORT.into()),
      BODHI_LOG_LEVEL => serde_yaml::Value::String(DEFAULT_LOG_LEVEL.to_string()),
      BODHI_LOG_STDOUT => serde_yaml::Value::Bool(DEFAULT_LOG_STDOUT),
      BODHI_EXEC_PATH => {
        // TODO: for development, below are the values
        // for native, need to get it from tauri
        // for container, need to set a convention
        let exec_path = format!(
          "{}/{}/{}",
          llama_server_proc::BUILD_TARGET,
          llama_server_proc::DEFAULT_VARIANT,
          llama_server_proc::EXEC_NAME
        );
        serde_yaml::Value::String(exec_path)
      }
      BODHI_EXEC_LOOKUP_PATH => {
        let lookup_path = std::env::current_dir()
          .unwrap_or_else(|err| {
            tracing::warn!("failed to get current directory. err: {err}");
            PathBuf::from(".")
              .canonicalize()
              .expect("failed to canonicalize current directory")
          })
          .display()
          .to_string()
          .replace('/', std::path::MAIN_SEPARATOR_STR);
        serde_yaml::Value::String(lookup_path)
      }
      BODHI_EXEC_VARIANT => {
        serde_yaml::Value::String(llama_server_proc::DEFAULT_VARIANT.to_string())
      }
      _ => serde_yaml::Value::Null,
    }
  }

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT => SettingMetadata::Number {
        min: 1025,
        max: 65535,
      },
      BODHI_LOG_LEVEL => SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
      BODHI_LOG_STDOUT => SettingMetadata::Boolean,
      BODHI_EXEC_PATH => {
        let mut options = Vec::new();
        for variant in llama_server_proc::BUILD_VARIANTS.iter() {
          let exec_path = format!(
            "{}/{}/{}",
            llama_server_proc::BUILD_TARGET,
            variant,
            llama_server_proc::EXEC_NAME
          );
          options.push(exec_path);
        }
        SettingMetadata::option(options)
      }
      _ => SettingMetadata::String,
    }
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
  use crate::{
    get_setting, set_setting, test_utils::EnvWrapperStub, DefaultSettingService, MockEnvWrapper,
    SettingService,
  };
  use mockall::predicate::eq;
  use objs::test_utils::temp_dir;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use std::{collections::HashMap, sync::Arc};
  use tempfile::TempDir;

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct TestConfig {
    name: String,
    value: i32,
  }

  #[rstest]
  fn test_setting_service_read_from_file_if_env_not_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(
      &path,
      r#"
TEST_KEY: file_value
TEST_NUMBER: 8080
"#,
    )?;
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new(Arc::new(env_wrapper), path.clone());
    assert_eq!(
      Some("file_value".to_string()),
      service.get_setting("TEST_KEY"),
    );
    assert_eq!(Some("8080".to_string()), service.get_setting("TEST_NUMBER"),);
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
