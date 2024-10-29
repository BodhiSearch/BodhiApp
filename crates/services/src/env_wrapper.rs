use std::{
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
pub struct DefaultEnvWrapper {}

impl EnvWrapper for DefaultEnvWrapper {
  fn var(&self, key: &str) -> Result<String, VarError> {
    std::env::var(key)
  }

  fn home_dir(&self) -> Option<PathBuf> {
    dirs::home_dir()
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
