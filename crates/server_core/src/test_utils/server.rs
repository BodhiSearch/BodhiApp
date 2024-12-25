use futures::FutureExt;
use llama_server_proc::MockServer;
use rstest::fixture;

#[fixture]
pub fn mock_server() -> MockServer {
  let mut mock_server = MockServer::default();
  mock_server
    .expect_start()
    .return_once(|| async { Ok(()) }.boxed());
  mock_server
}
