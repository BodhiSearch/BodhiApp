use crate::{
  EnvService, EnvServiceError, EnvWrapper, BODHI_APP_TYPE, BODHI_FRONTEND_URL, BODHI_HOME,
  BODHI_HOST, BODHI_PORT, BODHI_SCHEME, HF_HOME, LOGS_DIR,
};
use objs::{test_utils::temp_dir, AppType, EnvType, LogLevel};
use rstest::fixture;
use std::{
  collections::HashMap,
  env::VarError,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};
use tempfile::TempDir;

const TEST_AUTH_URL: &str = "TEST_AUTH_URL";
const TEST_AUTH_REALM: &str = "TEST_AUTH_REALM";

pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}

#[fixture]
pub fn test_env_service(
  #[default(HashMap::new())] envs: HashMap<String, String>,
) -> EnvServiceStub {
  EnvServiceStub::new(envs)
}

#[derive(Debug, Clone, Default)]
pub struct EnvServiceStub {
  envs: Arc<RwLock<HashMap<String, String>>>,
}

impl EnvServiceStub {
  pub fn new(envs: HashMap<String, String>) -> Self {
    Self {
      envs: Arc::new(RwLock::new(envs)),
    }
  }

  pub fn with_env(self, key: &str, value: &str) -> Self {
    self
      .envs
      .write()
      .unwrap()
      .insert(key.to_string(), value.to_string());
    self
  }
}

impl EnvService for EnvServiceStub {
  fn env_type(&self) -> EnvType {
    EnvType::Development
  }

  fn app_type(&self) -> AppType {
    match self.envs.read().unwrap().get(BODHI_APP_TYPE) {
      Some(app_type) => AppType::try_from(app_type.as_str()).unwrap_or(AppType::Container),
      None => AppType::Container,
    }
  }

  fn version(&self) -> String {
    "0.0.0".to_string()
  }

  fn bodhi_home(&self) -> PathBuf {
    match self.envs.read().unwrap().get(BODHI_HOME) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/bodhi"),
    }
  }

  fn hf_home(&self) -> PathBuf {
    match self.envs.read().unwrap().get(HF_HOME) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/hf"),
    }
  }

  fn logs_dir(&self) -> PathBuf {
    match self.envs.read().unwrap().get(LOGS_DIR) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/logs"),
    }
  }

  fn scheme(&self) -> String {
    match self.envs.read().unwrap().get(BODHI_SCHEME) {
      Some(path) => path.to_string(),
      None => "http".to_string(),
    }
  }

  fn host(&self) -> String {
    match self.envs.read().unwrap().get(BODHI_HOST) {
      Some(path) => path.to_string(),
      None => "localhost".to_string(),
    }
  }

  fn port(&self) -> u16 {
    match self.envs.read().unwrap().get(BODHI_PORT) {
      Some(path) => path.parse::<u16>().unwrap(),
      None => 1135,
    }
  }

  fn frontend_url(&self) -> String {
    match self.envs.read().unwrap().get(BODHI_FRONTEND_URL) {
      Some(path) => path.to_string(),
      None => self.server_url(),
    }
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join("test.db")
  }

  fn list(&self) -> HashMap<String, String> {
    self.envs.read().unwrap().clone()
  }

  fn auth_url(&self) -> String {
    match self.envs.read().unwrap().get(TEST_AUTH_URL) {
      Some(path) => path.to_string(),
      None => "http://id.localhost".to_string(),
    }
  }

  fn auth_realm(&self) -> String {
    match self.envs.read().unwrap().get(TEST_AUTH_REALM) {
      Some(path) => path.to_string(),
      None => "test-realm".to_string(),
    }
  }

  fn exec_path(&self) -> String {
    "/tmp/library-path.dylib".to_string()
  }

  fn set_setting(&self, _key: &str, _value: &str) -> Result<(), EnvServiceError> {
    Ok(())
  }

  fn exec_lookup_path(&self) -> String {
    "/tmp".to_string()
  }

  fn log_level(&self) -> LogLevel {
    LogLevel::Warn
  }

  fn encryption_key(&self) -> Option<String> {
    None
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
