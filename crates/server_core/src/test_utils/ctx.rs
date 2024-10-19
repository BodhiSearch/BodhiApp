use std::path::PathBuf;

use llamacpp_rs::MockServerContext;
use mockall::predicate::{always, eq};
use rstest::fixture;

#[fixture]
pub fn mock_server_ctx() -> MockServerContext {
  let mut mock_server_ctx = MockServerContext::default();
  mock_server_ctx
    .expect_load_library()
    .with(eq(PathBuf::from("/tmp/test_library.dylib")))
    .return_once(|_| Ok(()));
  mock_server_ctx
    .expect_disable_logging()
    .return_once(|| Ok(()));
  mock_server_ctx
    .expect_create_context()
    .with(always())
    .return_once(|_| Ok(()));
  mock_server_ctx.expect_init().return_once(|| Ok(()));
  mock_server_ctx
    .expect_start_event_loop()
    .return_once(|| Ok(()));
  mock_server_ctx.expect_stop().return_once(|| Ok(()));
  mock_server_ctx
}
