use std::{env::VarError, path::PathBuf};

pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}

mockall::mock! {
  pub EnvService {
    pub fn new() -> Self;

    pub fn var(&self, key: &str) -> Result<String, VarError>;

    pub fn home_dir(&self) -> Option<PathBuf>;
  }
}
