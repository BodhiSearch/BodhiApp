use super::{bodhi_home, TEST_REPO};
use rstest::fixture;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

pub struct ConfigDirs(pub TempDir, pub PathBuf, pub &'static str);

#[fixture]
pub fn config_dirs(bodhi_home: TempDir) -> ConfigDirs {
  let repo_dir = TEST_REPO.replace('/', "--");
  let repo_dir = format!("configs--{repo_dir}");
  let repo_dir = bodhi_home.path().join(repo_dir);
  fs::create_dir_all(repo_dir.clone()).unwrap();
  ConfigDirs(bodhi_home, repo_dir, TEST_REPO)
}
