use super::copy_test_dir;
use rstest::fixture;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

#[fixture]
pub fn temp_home() -> TempDir {
  tempdir().expect("Failed to create a temporary directory")
}

#[fixture]
pub fn temp_hf_home(temp_home: TempDir) -> TempDir {
  let dst_path = temp_home.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_home
}

#[fixture]
pub fn hf_cache(temp_hf_home: TempDir) -> (TempDir, PathBuf) {
  let hf_cache = temp_hf_home
    .path()
    .to_path_buf()
    .join("huggingface")
    .join("hub");
  (temp_hf_home, hf_cache)
}
