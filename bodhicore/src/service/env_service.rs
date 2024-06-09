use std::{env::VarError, path::PathBuf};

#[allow(unused)]
pub(crate) struct EnvWrapper {}

#[allow(unused)]
impl EnvWrapper {
  pub fn new() -> Self {
    EnvWrapper {}
  }

  pub fn var(&self, key: &str) -> Result<String, VarError> {
    std::env::var(key)
  }

  pub fn home_dir(&self) -> Option<PathBuf> {
    dirs::home_dir()
  }
}
