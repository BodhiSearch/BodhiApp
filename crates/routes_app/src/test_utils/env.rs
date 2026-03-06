use rstest::fixture;
use std::path::PathBuf;

fn load_env_test() {
  let env_path = PathBuf::from(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/resources/.env.test"
  ));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }
}

#[fixture]
pub fn setup_env() {
  load_env_test();
}
