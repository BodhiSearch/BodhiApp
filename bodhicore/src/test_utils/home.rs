use crate::server::BODHI_HOME;
use rstest::fixture;
use std::env;
use tempfile::TempDir;

#[fixture]
pub fn bodhi_home() -> TempDir {
  let bodhi_home = tempfile::Builder::new()
    .prefix("bodhi_home")
    .tempdir()
    .unwrap();
  env::set_var(BODHI_HOME, format!("{}", bodhi_home.path().display()));
  bodhi_home
}
