//! OAuth external token tests using ExternalTokenSimulator.
//!
//! Validates that external OAuth tokens (simulated via cache bypass) are correctly
//! handled by the auth middleware for MCP endpoints.

mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::Value;
use services::{
  test_utils::TEST_TENANT_ID, ApprovedResources, ApprovedResourcesV1, McpGrant, McpRequest,
  McpServerRequest, ModelGrant,
};
use utils::{start_test_live_server, ExternalTokenSimulator};

/// The review_url returned by POST /bodhi/v1/apps/request-access reflects the request's
/// Host (here reqwest's loopback 127.0.0.1) rather than the bind host (0.0.0.0), so the
/// link the app opens is same-origin with the caller. Anonymous endpoint — no token needed.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_create_access_request_review_url_reflects_request_host() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let response = client
    .post(format!("{}/bodhi/v1/apps/request-access", server.base_url))
    .json(&serde_json::json!({
      "app_client_id": "app-live-1",
      "requested_role": "scope_user_user",
      "requested": { "version": "1", "mcp_servers": [] }
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: Value = response.json().await?;
  let id = body["id"].as_str().expect("id present");
  let review_url = body["review_url"].as_str().expect("review_url present");
  assert_eq!(
    format!(
      "http://127.0.0.1:51135/ui/apps/access-requests/review?id={}",
      id
    ),
    review_url
  );

  server.handle.shutdown().await?;
  Ok(())
}

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

/// Approved-app grants flow end-to-end through real HTTP enforcement.
///
/// Seeds two MCP instances owned by the external user the simulator resolves to
/// (sub `test-external-user`, tenant `TEST_TENANT_ID`), then mints an external
/// token whose approved grants list only one of them. `GET /bodhi/v1/apps/mcps`
/// must return 200 with a list containing the granted instance but NOT the
/// ungranted one — proving grants flow: cache -> AuthContext::ExternalApp{grants}
/// -> AccessPolicy::Grants -> mcp_listable filtering, over real TCP.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_grants_filter_apps_mcps_list() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;

  // Seed an MCP server + two instances owned by the external user / tenant the
  // simulator resolves to, directly via the live server's app_service.
  let mcp_service = server.app_service.mcp_service();
  let mcp_server = mcp_service
    .create_mcp_server(
      TEST_TENANT_ID,
      "test-external-user",
      McpServerRequest {
        url: "https://mcp.grant-test.example.com/mcp".to_string(),
        name: "Grant Test Server".to_string(),
        description: None,
        enabled: true,
        auth_config: None,
      },
    )
    .await?;

  let make_request = |name: &str, slug: &str| McpRequest {
    name: name.to_string(),
    slug: slug.to_string(),
    mcp_server_id: Some(mcp_server.id.clone()),
    description: None,
    enabled: true,
    auth_type: Default::default(),
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  };

  let granted = mcp_service
    .create(
      TEST_TENANT_ID,
      "test-external-user",
      make_request("Granted MCP", "granted"),
    )
    .await?;
  let granted_id = granted.id.clone();

  let ungranted = mcp_service
    .create(
      TEST_TENANT_ID,
      "test-external-user",
      make_request("Ungranted MCP", "ungranted"),
    )
    .await?;
  let ungranted_id = ungranted.id.clone();

  // Mint an external token whose approved grants list only `granted_id`.
  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let grants = ApprovedResources::V1(ApprovedResourcesV1 {
    models_list: false,
    models_access: ModelGrant::Specific { ids: vec![] },
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific {
      ids: vec![granted_id.clone()],
    },
  });
  let bearer_token = simulator.create_token_with_grants(
    Some("scope_user_user"),
    "test-external-app",
    Some(grants),
  )?;

  let client = reqwest::Client::new();
  let response = client
    .get(format!("{}/bodhi/v1/apps/mcps", server.base_url))
    .header("Authorization", format!("Bearer {}", bearer_token))
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    response.status(),
    "External OAuth token with approved grants should access apps mcps list endpoint"
  );

  let body: Value = response.json().await?;
  let ids: Vec<String> = body["mcps"]
    .as_array()
    .expect("apps mcps response must have an mcps array")
    .iter()
    .map(|m| m["id"].as_str().expect("mcp must have id").to_string())
    .collect();

  assert_eq!(
    vec![granted_id.clone()],
    ids,
    "Grant-filtered list should contain only the granted MCP instance"
  );
  assert_eq!(
    true,
    ids.contains(&granted_id),
    "Granted MCP instance must appear in the list"
  );
  assert_eq!(
    false,
    ids.contains(&ungranted_id),
    "Ungranted MCP instance must NOT appear in the list"
  );

  server.handle.shutdown().await?;
  Ok(())
}
