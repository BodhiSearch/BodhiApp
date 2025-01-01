use rstest::fixture;
use std::process::Command;

pub fn exec_py_script(cwd: &str, script: &str) {
  let output = Command::new("python")
    .arg(script)
    .current_dir(cwd)
    .output()
    .expect("Failed to execute Python script");

  if !output.status.success() {
    panic!(
      "Python script {}/{} failed with status: {}, stderr: {}",
      cwd,
      script,
      output.status,
      String::from_utf8_lossy(&output.stderr)
    );
  }
}

#[fixture]
#[once]
pub fn generate_test_data_gguf_metadata() -> () {
  exec_py_script(
    env!("CARGO_MANIFEST_DIR"),
    "tests/scripts/test_data_gguf_metadata.py",
  );
}

#[fixture]
#[once]
pub fn generate_test_data_chat_template() -> () {
  exec_py_script(
    env!("CARGO_MANIFEST_DIR"),
    "tests/scripts/test_data_chat_template.py",
  );
}

#[fixture]
#[once]
pub fn generate_test_data_gguf_files() -> () {
  exec_py_script(
    env!("CARGO_MANIFEST_DIR"),
    "tests/scripts/test_data_gguf_files.py",
  );
}
