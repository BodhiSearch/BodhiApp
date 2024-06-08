use std::{env::VarError, path::PathBuf};

#[allow(unused)]
pub(crate) struct EnvService {}

#[allow(unused)]
impl EnvService {
  pub fn new() -> Self {
    EnvService {}
  }

  pub fn var(&self, key: &str) -> Result<String, VarError> {
    std::env::var(key)
  }

  pub fn home_dir(&self) -> Option<PathBuf> {
    dirs::home_dir()
  }
}
