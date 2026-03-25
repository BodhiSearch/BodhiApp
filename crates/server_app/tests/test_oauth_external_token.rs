//! OAuth external token tests using ExternalTokenSimulator.
//!
//! Validates that external OAuth tokens (simulated via cache bypass) are correctly
//! handled by the auth middleware for MCP endpoints.

mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use utils::{start_test_live_server, ExternalTokenSimulator};

/// External token with approved role can access GET /bodhi/v1/apps/mcps
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_role_can_list_mcps() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let bearer_token =
    simulator.create_token_with_role(Some("scope_user_user"), "test-external-app")?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/apps/mcps", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    response.status(),
    "External OAuth token with approved role should access apps mcps list endpoint"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// External token without approved role is rejected
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_without_role_is_rejected() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let bearer_token = simulator.create_token_with_role(None, "test-external-app")?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/apps/mcps", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token without approved role should be rejected"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// External token is rejected on session-only endpoints (GET /mcps/{id})
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_rejected_on_session_only_get() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let bearer_token =
    simulator.create_token_with_role(Some("scope_user_user"), "test-external-app")?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/mcps/some-id", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token should be rejected on session-only endpoint GET /mcps/{{id}}"
  );

  server.handle.shutdown().await?;
  Ok(())
}
