use super::{build_temp_dir, copy_test_dir};
use rstest::fixture;
use tempfile::TempDir;

#[fixture]
pub fn temp_dir() -> TempDir {
  build_temp_dir()
}

#[fixture]
pub fn temp_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

#[fixture]
pub fn empty_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  std::fs::create_dir_all(&dst_path).unwrap();
  temp_dir
}
