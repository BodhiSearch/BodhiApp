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
