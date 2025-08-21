use crate::{
  EnvWrapper, SettingService, SettingServiceError, SettingsChangeListener, BODHI_APP_TYPE,
  BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_NAME,
  BODHI_EXEC_VARIANT, BODHI_EXEC_VARIANTS, BODHI_HOME, BODHI_HOST, BODHI_KEEP_ALIVE_SECS,
  BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT, BODHI_PORT, BODHI_SCHEME, BODHI_VERSION, HF_HOME,
};
use llama_server_proc::{BUILD_VARIANTS, DEFAULT_VARIANT, EXEC_NAME};
use objs::{test_utils::temp_dir, Setting, SettingInfo, SettingMetadata, SettingSource};
use rstest::fixture;
use std::{
  collections::HashMap,
  env::VarError,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};
use tempfile::TempDir;

pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}

#[fixture]
pub fn test_setting_service(
  #[default(HashMap::new())] envs: HashMap<String, String>,
) -> SettingServiceStub {
  SettingServiceStub::new(envs)
}

#[derive(Debug, Clone)]
pub struct SettingServiceStub {
  settings: Arc<RwLock<HashMap<String, serde_yaml::Value>>>,
  envs: HashMap<String, String>,
}

impl SettingServiceStub {
  pub fn new_with_env(envs: HashMap<String, String>, settings: HashMap<String, String>) -> Self {
    let settings = Self::to_settings_value(settings);
    Self {
      settings: Arc::new(RwLock::new(settings)),
      envs,
    }
  }

  pub fn new(settings: HashMap<String, String>) -> Self {
    Self::new_with_env(HashMap::new(), settings)
  }

  pub fn with_settings(self, settings: HashMap<String, String>) -> Self {
    let settings = Self::to_settings_value(settings);
    for (key, value) in settings {
      self.set_setting_with_source(key.as_str(), &value, SettingSource::SettingsFile);
    }
    self
  }

  fn to_settings_value(settings: HashMap<String, String>) -> HashMap<String, serde_yaml::Value> {
    settings
      .iter()
      .map(|(key, value)| {
        let metadata = get_metadata(key);
        let value = metadata.parse(serde_yaml::Value::String(value.to_string()));
        (key.to_string(), value)
      })
      .collect::<HashMap<String, serde_yaml::Value>>()
  }
}

impl Default for SettingServiceStub {
  fn default() -> Self {
    let settings = HashMap::from([
      ("HOME".to_string(), "/tmp/home".to_string()),
      (BODHI_ENV_TYPE.to_string(), "development".to_string()),
      (BODHI_APP_TYPE.to_string(), "container".to_string()),
      (BODHI_VERSION.to_string(), "0.0.0".to_string()),
      (
        BODHI_AUTH_URL.to_string(),
        "http://id.localhost".to_string(),
      ),
      (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
      (BODHI_HOME.to_string(), "/tmp/bodhi".to_string()),
      (BODHI_LOGS.to_string(), "/tmp/logs".to_string()),
      (HF_HOME.to_string(), "/tmp/hf".to_string()),
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "localhost".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
      (BODHI_LOG_LEVEL.to_string(), "warn".to_string()),
      (BODHI_LOG_STDOUT.to_string(), "true".to_string()),
      (BODHI_ENCRYPTION_KEY.to_string(), "testkey".to_string()),
      (BODHI_EXEC_VARIANT.to_string(), DEFAULT_VARIANT.to_string()),
      (
        BODHI_EXEC_VARIANTS.to_string(),
        BUILD_VARIANTS.join(",").to_string(),
      ),
      (BODHI_EXEC_NAME.to_string(), EXEC_NAME.to_string()),
    ]);
    Self::new(settings)
  }
}

impl SettingService for SettingServiceStub {
  fn load(&self, _path: &Path) {}

  fn load_default_env(&self) {}

  fn home_dir(&self) -> Option<PathBuf> {
    self
      .settings
      .read()
      .unwrap()
      .get("HOME")
      .map(|home| PathBuf::from(home.as_str().unwrap()))
  }

  fn list(&self) -> Vec<SettingInfo> {
    self
      .settings
      .read()
      .unwrap()
      .iter()
      .map(|(key, value)| SettingInfo {
        key: key.clone(),
        current_value: value.clone(),
        default_value: serde_yaml::Value::Null,
        source: SettingSource::Environment,
        metadata: self.get_setting_metadata(key),
      })
      .collect()
  }

  fn get_default_value(&self, _key: &str) -> Option<serde_yaml::Value> {
    None
  }

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    get_metadata(key)
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.envs.get(key).cloned()
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<serde_yaml::Value>, SettingSource) {
    let lock = self.settings.read().unwrap();
    match lock.get(key).cloned() {
      Some(value) => (Some(value), SettingSource::SettingsFile),
      None if key.starts_with("BODHI_PUBLIC_") => (
        Some(
          lock
            .get(&key.replace("BODHI_PUBLIC_", "BODHI_"))
            .cloned()
            .unwrap(),
        ),
        SettingSource::Default,
      ),
      None => panic!("Setting with key: {key} not found"),
    }
  }

  fn set_setting_with_source(&self, key: &str, value: &serde_yaml::Value, _source: SettingSource) {
    let mut lock = self.settings.write().unwrap();
    lock.insert(key.to_string(), value.clone());
  }

  fn delete_setting(&self, key: &str) -> Result<(), SettingServiceError> {
    let mut lock = self.settings.write().unwrap();
    lock.remove(key);
    Ok(())
  }

  fn add_listener(&self, _listener: Arc<dyn SettingsChangeListener>) {}
}

fn get_metadata(key: &str) -> SettingMetadata {
  match key {
    BODHI_PORT => SettingMetadata::Number { min: 1, max: 65535 },
    BODHI_LOG_STDOUT => SettingMetadata::Boolean,
    BODHI_KEEP_ALIVE_SECS => SettingMetadata::Number {
      min: 300,
      max: 86400,
    },
    _ => SettingMetadata::String,
  }
}

#[derive(Debug)]
pub struct EnvWrapperStub {
  envs: Arc<RwLock<HashMap<String, String>>>,
  temp_dir: TempDir,
}

impl EnvWrapperStub {
  pub fn new(envs: HashMap<String, String>) -> Self {
    let temp_dir = temp_dir();
    Self {
      envs: Arc::new(RwLock::new(envs)),
      temp_dir,
    }
  }
}

impl EnvWrapper for EnvWrapperStub {
  fn var(&self, key: &str) -> Result<String, VarError> {
    match self.envs.read().unwrap().get(key) {
      Some(path) => Ok(path.to_string()),
      None => Err(VarError::NotPresent),
    }
  }

  fn home_dir(&self) -> Option<PathBuf> {
    match self.envs.read().unwrap().get("HOME") {
      Some(path) => Some(PathBuf::from(path)),
      None => Some(self.temp_dir.path().to_path_buf()),
    }
  }

  fn load(&self, _path: &Path) {
    //
  }
}

pub fn bodhi_home_setting(path: &Path, source: SettingSource) -> Setting {
  Setting {
    key: BODHI_HOME.to_string(),
    value: serde_yaml::Value::String(path.display().to_string()),
    source,
    metadata: SettingMetadata::String,
  }
}
