mod utils;

use crate::utils::{live_server, TestServerHandle};
use axum::http::StatusCode;
use reqwest::Client;
use rstest::rstest;
use serial_test::serial;
use std::time::Duration;

#[rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial(live)]
async fn test_live_api_ping(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    temp_cache_dir: _temp_cache_dir,
    host,
    port,
    handle,
  } = live_server?;
  let ping_endpoint = format!("http://{host}:{port}/ping");
  let client = Client::new();
  let response = client.get(ping_endpoint).send().await?;
  handle.shutdown().await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
