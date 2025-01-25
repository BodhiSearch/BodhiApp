use crate::{asref_impl, EnvWrapper, BODHI_HOME};
use objs::{
  impl_error_from, AppError, ErrorType, IoError, SerdeYamlError, SettingInfo, SettingMetadata,
  SettingSource,
};
use serde_yaml::Value;
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};

pub const HF_HOME: &str = "HF_HOME";
pub const BODHI_LOGS: &str = "BODHI_LOGS";
pub const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
pub const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";
pub const BODHI_SCHEME: &str = "BODHI_SCHEME";
pub const BODHI_HOST: &str = "BODHI_HOST";
pub const BODHI_PORT: &str = "BODHI_PORT";
pub const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
pub const BODHI_EXEC_VARIANT: &str = "BODHI_EXEC_VARIANT";
pub const BODHI_KEEP_ALIVE_SECS: &str = "BODHI_KEEP_ALIVE_SECS";

pub const DEFAULT_SCHEME: &str = "http";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 1135;
pub const DEFAULT_PORT_STR: &str = "1135";
pub const DEFAULT_LOG_LEVEL: &str = "warn";
pub const DEFAULT_LOG_STDOUT: bool = false;
pub const DEFAULT_KEEP_ALIVE_SECS: i64 = 300;

pub const SETTINGS_YAML: &str = "settings.yaml";

pub const SETTING_VARS: &[&str] = &[
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  HF_HOME,
  BODHI_SCHEME,
  BODHI_HOST,
  BODHI_PORT,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_EXEC_VARIANT,
  BODHI_KEEP_ALIVE_SECS,
];

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SettingsChangeListener: std::fmt::Debug + Send + Sync {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  );
}

impl SettingsChangeListener for Arc<dyn SettingsChangeListener> {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  ) {
    self
      .as_ref()
      .on_change(key, prev_value, prev_source, new_value, new_source)
  }
}

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
  #[error("invalid_source")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSource,
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

  fn load_default_env(&self);

  fn home_dir(&self) -> Option<PathBuf>;

  fn list(&self) -> Vec<SettingInfo>;

  fn get_default_value(&self, key: &str) -> Option<Value>;

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata;

  fn get_env(&self, key: &str) -> Option<String>;

  fn get_setting(&self, key: &str) -> Option<String> {
    match self.get_setting_value(key) {
      Some(value) => match value {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        _ => None,
      },
      None => None,
    }
  }

  fn get_setting_value(&self, key: &str) -> Option<Value> {
    self.get_setting_value_with_source(key).0
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource);

  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource);

  fn set_setting(&self, key: &str, value: &str) {
    self.set_setting_value(key, &Value::String(value.to_owned()))
  }

  fn set_setting_value(&self, key: &str, value: &Value) {
    self.set_setting_with_source(key, value, SettingSource::SettingsFile)
  }

  fn set_default(&self, key: &str, value: &Value) {
    self.set_setting_with_source(key, value, SettingSource::Default)
  }

  fn delete_setting(&self, key: &str) -> Result<()>;

  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>);
}

#[derive(Debug)]
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  path: PathBuf,
  settings_lock: RwLock<()>,
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  cmd_lines: RwLock<HashMap<String, Value>>,
}

impl DefaultSettingService {
  fn new(env_wrapper: Arc<dyn EnvWrapper>, path: PathBuf) -> DefaultSettingService {
    Self {
      env_wrapper,
      path,
      settings_lock: RwLock::new(()),
      defaults: RwLock::new(HashMap::new()),
      listeners: RwLock::new(Vec::new()),
      cmd_lines: RwLock::new(HashMap::new()),
    }
  }

  pub fn new_with_defaults(
    env_wrapper: Arc<dyn EnvWrapper>,
    bodhi_home: &Path,
    source: SettingSource,
    settings_file: PathBuf,
  ) -> Result<Self> {
    let service = Self::new(env_wrapper, settings_file);
    service.init_defaults(bodhi_home, source);
    Ok(service)
  }

  fn init_defaults(&self, bodhi_home: &Path, source: SettingSource) {
    self.with_defaults_write_lock(|defaults| {
      if source == SettingSource::Default {
        defaults.insert(
          BODHI_HOME.to_string(),
          Value::String(bodhi_home.display().to_string()),
        );
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(bodhi_home.join("logs").display().to_string()),
        );
      } else if let Some(home_dir) = self.home_dir() {
        let default_bodhi_home = home_dir.join(".cache").join("bodhi");
        defaults.insert(
          BODHI_HOME.to_string(),
          Value::String(default_bodhi_home.display().to_string()),
        );
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(default_bodhi_home.join("logs").display().to_string()),
        );
      }
      if let Some(home_dir) = self.home_dir() {
        defaults.insert(
          HF_HOME.to_string(),
          Value::String(
            home_dir
              .join(".cache")
              .join("huggingface")
              .display()
              .to_string(),
          ),
        );
      }
      defaults.insert(
        BODHI_SCHEME.to_string(),
        Value::String(DEFAULT_SCHEME.to_string()),
      );
      defaults.insert(
        BODHI_HOST.to_string(),
        Value::String(DEFAULT_HOST.to_string()),
      );
      defaults.insert(BODHI_PORT.to_string(), Value::Number(DEFAULT_PORT.into()));
      defaults.insert(
        BODHI_LOG_LEVEL.to_string(),
        Value::String(DEFAULT_LOG_LEVEL.to_string()),
      );
      defaults.insert(
        BODHI_LOG_STDOUT.to_string(),
        Value::Bool(DEFAULT_LOG_STDOUT),
      );
      defaults.insert(
        BODHI_EXEC_VARIANT.to_string(),
        Value::String(llama_server_proc::DEFAULT_VARIANT.to_string()),
      );
      defaults.insert(
        BODHI_KEEP_ALIVE_SECS.to_string(),
        Value::Number(DEFAULT_KEEP_ALIVE_SECS.into()),
      );
    });
  }

  pub fn with_settings_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&serde_yaml::Mapping) -> R,
  {
    let _guard = self.settings_lock.read().unwrap();
    if !self.path.exists() {
      return f(&serde_yaml::Mapping::new());
    }
    let contents = fs::read_to_string(&self.path).unwrap_or_else(|_| String::new());
    let settings: serde_yaml::Mapping =
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new());
    f(&settings)
  }

  pub fn with_settings_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut serde_yaml::Mapping),
  {
    let _guard = self.settings_lock.write().unwrap();
    let mut settings = if !self.path.exists() {
      serde_yaml::Mapping::new()
    } else {
      let contents = fs::read_to_string(&self.path).unwrap_or_else(|_| String::new());
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new())
    };
    f(&mut settings);
    let contents = serde_yaml::to_string(&settings).unwrap();
    fs::write(&self.path, contents).unwrap();
  }

  pub fn with_defaults_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let defaults = self.defaults.read().unwrap();
    f(&defaults)
  }

  pub fn with_defaults_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut defaults = self.defaults.write().unwrap();
    f(&mut defaults);
  }

  pub fn with_cmd_lines_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let cmd_lines = self.cmd_lines.read().unwrap();
    f(&cmd_lines)
  }

  pub fn with_cmd_lines_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut cmd_lines = self.cmd_lines.write().unwrap();
    f(&mut cmd_lines);
  }

  fn notify_listeners(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  ) {
    let lock = self.listeners.read().unwrap();
    for listener in lock.iter() {
      listener.on_change(key, prev_value, prev_source, new_value, new_source);
    }
  }
}

impl SettingService for DefaultSettingService {
  fn load(&self, path: &Path) {
    self.env_wrapper.load(path);
  }

  fn load_default_env(&self) {
    let bodhi_home = self
      .get_setting(BODHI_HOME)
      .expect("BODHI_HOME should be set");
    let env_file = PathBuf::from(bodhi_home).join(".env");
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

  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource) {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    match source {
      SettingSource::CommandLine => {
        self.with_cmd_lines_write_lock(|cmd_lines| {
          cmd_lines.insert(key.to_string(), value.clone());
        });
      }
      SettingSource::Environment => {
        tracing::error!("SettingSource::Environment is not supported for override");
      }
      SettingSource::SettingsFile => {
        self.with_settings_write_lock(|settings| {
          settings.insert(key.into(), value.clone());
        });
        let (cur_value, cur_source) = self.get_setting_value_with_source(key);
        self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
      }
      SettingSource::Default => {
        self.with_defaults_write_lock(|defaults| {
          defaults.insert(key.to_string(), value.clone());
        });
      }
    }
  }

  fn delete_setting(&self, key: &str) -> Result<()> {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    self.with_settings_write_lock(|settings| {
      settings.remove(key);
    });
    let (cur_value, cur_source) = self.get_setting_value_with_source(key);
    self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
    Ok(())
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
    let metadata = self.get_setting_metadata(key);
    let result = self.with_cmd_lines_read_lock(|cmd_lines| cmd_lines.get(key).cloned());
    if let Some(value) = result {
      return (Some(value), SettingSource::CommandLine);
    }
    if let Ok(value) = self.env_wrapper.var(key) {
      let value = metadata.parse(Value::String(value));
      return (Some(value), SettingSource::Environment);
    }
    let result = self.with_settings_read_lock(|settings| settings.get(key).cloned());
    result
      .map(|value| (Some(metadata.parse(value)), SettingSource::SettingsFile))
      .unwrap_or((self.get_default_value(key), SettingSource::Default))
  }

  fn list(&self) -> Vec<SettingInfo> {
    SETTING_VARS
      .iter()
      .map(|key| {
        let (current_value, source) = self.get_setting_value_with_source(key);
        let metadata = self.get_setting_metadata(key);
        let current_value = current_value.map(|value| metadata.parse(value));

        SettingInfo {
          key: key.to_string(),
          current_value: current_value.unwrap_or(Value::Null),
          default_value: self.get_default_value(key).unwrap_or(Value::Null),
          source,
          metadata,
        }
      })
      .collect()
  }

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT => SettingMetadata::Number { min: 1, max: 65535 },
      BODHI_LOG_LEVEL => SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
      BODHI_LOG_STDOUT => SettingMetadata::Boolean,
      BODHI_EXEC_VARIANT => {
        let mut options = Vec::new();
        for variant in llama_server_proc::BUILD_VARIANTS.iter() {
          options.push(variant.to_string());
        }
        SettingMetadata::option(options)
      }
      BODHI_KEEP_ALIVE_SECS => SettingMetadata::Number {
        min: 300,
        max: 86400,
      },
      _ => SettingMetadata::String,
    }
  }

  fn get_default_value(&self, key: &str) -> Option<Value> {
    self.with_defaults_read_lock(|defaults| match key {
      BODHI_HOME => match defaults.get(BODHI_HOME).cloned() {
        Some(value) => Some(value),
        None => {
          let result = self
            .home_dir()
            .map(|home_dir| home_dir.join(".cache").join("bodhi"))
            .map(|path| serde_yaml::Value::String(path.display().to_string()));
          result
        }
      },
      _ => defaults.get(key).cloned(),
    })
  }

  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>) {
    let mut listeners = self.listeners.write().unwrap();
    if !listeners
      .iter()
      .any(|existing| std::ptr::eq(existing.as_ref(), listener.as_ref()))
    {
      listeners.push(listener);
    }
  }
}

asref_impl!(SettingService, DefaultSettingService);

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::EnvWrapperStub, DefaultSettingService, MockEnvWrapper, MockSettingsChangeListener,
    SettingService, BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_HOST, BODHI_LOGS, BODHI_LOG_LEVEL,
    BODHI_LOG_STDOUT, BODHI_PORT, BODHI_SCHEME, DEFAULT_HOST, DEFAULT_LOG_LEVEL,
    DEFAULT_LOG_STDOUT, DEFAULT_PORT, DEFAULT_SCHEME, HF_HOME,
  };
  use mockall::predicate::eq;
  use objs::{test_utils::temp_dir, SettingSource};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_yaml::Value;
  use std::{collections::HashMap, fs::read_to_string, sync::Arc};
  use tempfile::TempDir;

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct TestConfig {
    name: String,
    value: i32,
  }

  #[rstest]
  fn test_setting_service_init_with_defaults(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let home_dir = temp_dir.path().join("home");
    let env_wrapper =
      EnvWrapperStub::new(maplit::hashmap! {"HOME".to_string() => home_dir.display().to_string()});
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      &temp_dir.path(),
      SettingSource::Environment,
      path.clone(),
    )?;
    for (key, expected) in [
      (
        BODHI_HOME,
        home_dir.join(".cache").join("bodhi").display().to_string(),
      ),
      (
        BODHI_LOGS,
        home_dir
          .join(".cache")
          .join("bodhi")
          .join("logs")
          .display()
          .to_string(),
      ),
      (
        HF_HOME,
        home_dir
          .join(".cache")
          .join("huggingface")
          .display()
          .to_string(),
      ),
      (BODHI_SCHEME, DEFAULT_SCHEME.to_string()),
      (BODHI_HOST, DEFAULT_HOST.to_string()),
      (BODHI_LOG_LEVEL, DEFAULT_LOG_LEVEL.to_string()),
      (
        BODHI_EXEC_VARIANT,
        llama_server_proc::DEFAULT_VARIANT.to_string(),
      ),
    ] {
      assert_eq!(
        expected,
        service.get_default_value(key).unwrap().as_str().unwrap()
      );
    }
    assert_eq!(
      DEFAULT_PORT as i64,
      service
        .get_default_value(BODHI_PORT)
        .unwrap()
        .as_i64()
        .unwrap()
    );
    assert_eq!(
      DEFAULT_LOG_STDOUT,
      service
        .get_default_value(BODHI_LOG_STDOUT)
        .unwrap()
        .as_bool()
        .unwrap()
    );
    Ok(())
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
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      temp_dir.path(),
      SettingSource::Environment,
      path.clone(),
    )?;
    assert_eq!(
      Some("file_value".to_string()),
      service.get_setting("TEST_KEY"),
    );
    assert_eq!(Some("8080".to_string()), service.get_setting("TEST_NUMBER"),);
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_default_if_not_set_in_env_or_file(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(
      &path,
      r#"
SOME_OTHER_KEY: value
"#,
    )?;
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new(Arc::new(env_wrapper), path.clone());
    service.set_default("SOME_KEY", &Value::String("default_value".to_string()));
    assert_eq!(
      Some("default_value".to_string()),
      service.get_setting("SOME_KEY"),
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
      .times(1)
      .return_const(Ok("env_value".to_string()));
    let service = DefaultSettingService::new(Arc::new(mock_env), path);
    assert_eq!(
      Some("env_value".to_string()),
      service.get_setting("TEST_KEY")
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_cmd_line_if_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: file_value")?;
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Ok("env_value".to_string()));
    let service = DefaultSettingService::new(Arc::new(mock_env), path);
    service.set_setting_with_source(
      "TEST_KEY",
      &serde_yaml::Value::String("cmdline-value".to_string()),
      SettingSource::CommandLine,
    );
    assert_eq!(
      Some("cmdline-value".to_string()),
      service.get_setting("TEST_KEY")
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_overriding_settings(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("test_value".to_string()))),
        eq(SettingSource::SettingsFile),
        eq(Some(Value::String("new_value".to_string()))),
        eq(SettingSource::SettingsFile),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.set_setting("TEST_KEY", "new_value");
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_deleting_settings(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::default();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
    service.set_default("TEST_KEY", &Value::String("default_value".to_string()));
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("test_value".to_string()))),
        eq(SettingSource::SettingsFile),
        eq(Some(Value::String("default_value".to_string()))),
        eq(SettingSource::Default),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.delete_setting("TEST_KEY")?;
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_env_set(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Ok("env_value".to_string()));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("env_value".to_string()))),
        eq(SettingSource::Environment),
        eq(Some(Value::String("env_value".to_string()))),
        eq(SettingSource::Environment),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.set_setting("TEST_KEY", "new_value");
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_setting_defaults(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .times(1)
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener.expect_on_change().never();
    service.add_listener(Arc::new(mock_listener));
    service.set_default("TEST_KEY", &Value::String("default_value".to_string()));
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
      service.set_setting("TEST_KEY", "test_value");
    }
    let contents = read_to_string(&path)?;
    assert_eq!("TEST_KEY: test_value\n", contents);

    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    {
      let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());
      assert_eq!(
        Some("test_value".to_string()),
        service.get_setting("TEST_KEY"),
      );
    }
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
    mock_env
      .expect_var()
      .with(eq("NON_EXISTENT_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone());

    service.set_setting("TEST_KEY", "test_value");
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
}
