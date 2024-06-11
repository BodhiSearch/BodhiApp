use std::{env::VarError, path::PathBuf};

#[derive(Debug, Clone, Default)]
pub struct EnvWrapper {}

impl EnvWrapper {
  pub fn var(&self, key: &str) -> Result<String, VarError> {
    std::env::var(key)
  }

  pub fn home_dir(&self) -> Option<PathBuf> {
    dirs::home_dir()
  }
}
