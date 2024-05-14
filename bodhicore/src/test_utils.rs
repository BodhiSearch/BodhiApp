use rstest::fixture;
use std::{env, fs, path::PathBuf};
use tempfile::TempDir;

use crate::server::BODHI_HOME;

static TEST_REPO: &str = "meta-llama/Meta-Llama-3-8B";

pub struct ConfigDirs(pub TempDir, pub PathBuf, pub &'static str);

#[fixture]
pub fn config_dirs(bodhi_home: TempDir) -> ConfigDirs {
  let repo_dir = TEST_REPO.replace('/', "--");
  let repo_dir = format!("configs--{repo_dir}");
  let repo_dir = bodhi_home.path().join(repo_dir);
  fs::create_dir_all(repo_dir.clone()).unwrap();
  ConfigDirs(bodhi_home, repo_dir, TEST_REPO)
}

#[fixture]
pub fn bodhi_home() -> TempDir {
  let bodhi_home = tempfile::Builder::new()
    .prefix("bodhi_home")
    .tempdir()
    .unwrap();
  env::set_var(BODHI_HOME, format!("{}", bodhi_home.path().display()));
  bodhi_home
}
