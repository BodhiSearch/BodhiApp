use super::{copy_test_dir, temp_dir};
use rstest::fixture;
use std::path::PathBuf;
use tempfile::TempDir;

#[fixture]
pub fn temp_hf_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_dir
}

#[fixture]
pub fn empty_hf_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("huggingface").join("hub");
  std::fs::create_dir_all(&dst_path).unwrap();
  temp_dir
}
