use super::copy_test_dir;
use rstest::fixture;
use tempfile::{tempdir, TempDir};

#[fixture]
pub fn temp_bodhi_home(temp_dir: TempDir) -> TempDir {
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

#[fixture]
pub fn temp_dir() -> TempDir {
  tempdir().expect("Failed to create a temporary directory")
}
