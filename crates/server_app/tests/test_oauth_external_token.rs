//! OAuth external token tests using ExternalTokenSimulator.
//!
//! Validates that external OAuth tokens (simulated via cache bypass) are correctly
//! handled by the auth middleware for toolset endpoints.

mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use utils::{start_test_live_server, ExternalTokenSimulator};

/// External token with scope_user_user can access GET /bodhi/v1/toolsets
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_scope_can_list_toolsets() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let bearer_token =
    simulator.create_token_with_scope("scope_user_user offline_access", "test-external-app")?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/toolsets", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    response.status(),
    "External OAuth token with scope_user_user should access toolsets list endpoint"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// External token without scope_user_user scope is rejected
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_without_scope_is_rejected() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  // Create token with only offline_access — no user scope
  let bearer_token = simulator.create_token_with_scope("offline_access", "test-external-app")?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/toolsets", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token without scope_user_user should be rejected"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// External token is rejected on session-only endpoints (GET /toolsets/{id})
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_rejected_on_session_only_get() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let bearer_token =
    simulator.create_token_with_scope("scope_user_user offline_access", "test-external-app")?;

  let client = reqwest::Client::new();
  // GET /toolsets/{id} is session-only (no OAuth or API tokens)
  let response = client
    .get(format!("{}/bodhi/v1/toolsets/some-id", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token should be rejected on session-only endpoint GET /toolsets/{{id}}"
  );

  server.handle.shutdown().await?;
  Ok(())
}
