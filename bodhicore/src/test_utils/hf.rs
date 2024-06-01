use super::copy_test_dir;
use rstest::fixture;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

#[fixture]
pub fn temp_hf_home() -> TempDir {
  let temp_dir = tempdir().expect("Failed to create a temporary directory");
  let dst_path = temp_dir.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_dir
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
