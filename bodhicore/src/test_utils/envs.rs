use crate::service::{
  EnvServiceFn, AUTH_REALM, AUTH_URL, BODHI_HOME, BODHI_HOST, BODHI_PORT, HF_HOME, LOGS_DIR,
};
use std::{
  collections::HashMap,
  env::VarError,
  fmt,
  path::PathBuf,
  sync::{Arc, Mutex},
};

pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}

mockall::mock! {
  pub EnvWrapper {
    pub fn new() -> Self;

    pub fn var(&self, key: &str) -> Result<String, VarError>;

    pub fn home_dir(&self) -> Option<PathBuf>;

    pub fn load_dotenv(&self);
  }

  impl std::fmt::Debug for EnvWrapper {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result;
  }

  impl Clone for EnvWrapper {
    fn clone(&self) -> Self;
  }
}

#[derive(Debug, Clone, Default)]
pub struct EnvServiceStub {
  envs: Arc<Mutex<HashMap<String, String>>>,
}

impl EnvServiceFn for EnvServiceStub {
  fn env_type(&self) -> String {
    "test".to_string()
  }

  fn is_production(&self) -> bool {
    false
  }

  fn version(&self) -> String {
    "0.0.0".to_string()
  }

  fn bodhi_home(&self) -> PathBuf {
    match self.envs.lock().unwrap().get(BODHI_HOME) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/bodhi"),
    }
  }

  fn hf_home(&self) -> PathBuf {
    match self.envs.lock().unwrap().get(HF_HOME) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/hf"),
    }
  }

  fn logs_dir(&self) -> PathBuf {
    match self.envs.lock().unwrap().get(LOGS_DIR) {
      Some(path) => PathBuf::from(path),
      None => PathBuf::from("/tmp/logs"),
    }
  }

  fn host(&self) -> String {
    match self.envs.lock().unwrap().get(BODHI_HOST) {
      Some(path) => path.to_string(),
      None => "localhost".to_string(),
    }
  }

  fn port(&self) -> u16 {
    match self.envs.lock().unwrap().get(BODHI_PORT) {
      Some(path) => path.parse::<u16>().unwrap(),
      None => 1135,
    }
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join("test.db")
  }

  fn list(&self) -> HashMap<String, String> {
    self.envs.lock().unwrap().clone()
  }

  fn auth_url(&self) -> String {
    match self.envs.lock().unwrap().get(AUTH_URL) {
      Some(path) => path.to_string(),
      None => "http://id.localhost".to_string(),
    }
  }

  fn auth_realm(&self) -> String {
    match self.envs.lock().unwrap().get(AUTH_REALM) {
      Some(path) => path.to_string(),
      None => "test-realm".to_string(),
    }
  }
}
