use std::{env::VarError, fmt, path::PathBuf};

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
