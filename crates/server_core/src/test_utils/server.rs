use futures::FutureExt;
use llama_server_proc::MockServer;
use rstest::fixture;
use services::test_utils::temp_dir;
use tempfile::TempDir;

#[fixture]
pub fn mock_server() -> MockServer {
  let mut mock_server = MockServer::default();
  mock_server
    .expect_start()
    .times(1)
    .return_once(|| async { Ok(()) }.boxed());
  mock_server
}

#[fixture]
pub fn bin_path(temp_dir: TempDir) -> TempDir {
  let cpu_exec_path = temp_dir
    .path()
    .join(llama_server_proc::BUILD_TARGET)
    .join(llama_server_proc::DEFAULT_VARIANT);
  std::fs::create_dir_all(&cpu_exec_path).expect("Failed to create directory");
  std::fs::write(cpu_exec_path.join(llama_server_proc::EXEC_NAME), "")
    .expect("Failed to write to file");
  temp_dir
}
