use std::{
  collections::HashMap,
  env::VarError,
  path::{Path, PathBuf},
};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EnvWrapper: Send + Sync + std::fmt::Debug {
  fn var(&self, key: &str) -> Result<String, VarError>;

  fn home_dir(&self) -> Option<PathBuf>;

  fn load(&self, path: &Path);
}

#[derive(Debug, Clone, Default)]
pub struct DefaultEnvWrapper {
  env_vars: HashMap<String, String>,
}

impl DefaultEnvWrapper {
  pub fn set_var(&mut self, key: &str, value: &str) {
    self.env_vars.insert(key.to_string(), value.to_string());
  }
}

impl EnvWrapper for DefaultEnvWrapper {
  fn var(&self, key: &str) -> Result<String, VarError> {
    // TODO: should check internal map first before checking the environment
    match std::env::var(key) {
      Ok(value) => Ok(value),
      Err(VarError::NotPresent) => match self.env_vars.get(key) {
        Some(value) => Ok(value.clone()),
        None => Err(VarError::NotPresent),
      },
      Err(err) => Err(err),
    }
  }

  fn home_dir(&self) -> Option<PathBuf> {
    match self.env_vars.get("HOME") {
      Some(path) => Some(PathBuf::from(path)),
      None => dirs::home_dir(),
    }
  }

  fn load(&self, envfile: &Path) {
    if let Err(err) = dotenv::from_path(envfile) {
      eprintln!(
        "error loading .env file. err: {}, path: {}",
        err,
        envfile.display()
      );
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow_trace::anyhow_trace;
  use pretty_assertions::assert_eq;
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[test]
  fn test_var_from_env() {
    let wrapper = DefaultEnvWrapper::default();
    std::env::set_var("TEST_KEY", "test_value");
    assert_eq!("test_value", wrapper.var("TEST_KEY").unwrap());
    std::env::remove_var("TEST_KEY");
  }

  #[test]
  fn test_var_from_wrapper() {
    let mut wrapper = DefaultEnvWrapper::default();
    wrapper.set_var("CUSTOM_KEY", "custom_value");
    assert_eq!("custom_value", wrapper.var("CUSTOM_KEY").unwrap());
  }

  #[test]
  fn test_var_not_found() {
    let wrapper = DefaultEnvWrapper::default();
    match wrapper.var("NONEXISTENT_KEY") {
      Err(VarError::NotPresent) => (),
      _ => panic!("Expected VarError::NotPresent"),
    }
  }

  #[test]
  fn test_home_dir() {
    let wrapper = DefaultEnvWrapper::default();
    assert!(wrapper.home_dir().is_some());
  }

  #[test]
  fn test_load_env_file() {
    let wrapper = DefaultEnvWrapper::default();
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "TEST_ENV_VAR=test_value").unwrap();
    wrapper.load(temp_file.path());
    assert_eq!("test_value", std::env::var("TEST_ENV_VAR").unwrap());
    std::env::remove_var("TEST_ENV_VAR");
  }

  #[test]
  fn test_set_var_overwrites() {
    let mut wrapper = DefaultEnvWrapper::default();
    wrapper.set_var("OVERWRITE_KEY", "first_value");
    assert_eq!("first_value", wrapper.var("OVERWRITE_KEY").unwrap());
    wrapper.set_var("OVERWRITE_KEY", "second_value");
    assert_eq!("second_value", wrapper.var("OVERWRITE_KEY").unwrap());
  }
}
