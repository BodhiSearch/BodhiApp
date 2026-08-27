//! OAuth external token tests using ExternalTokenSimulator.
//!
//! Validates that external OAuth tokens (simulated via cache bypass) are correctly
//! handled by the auth middleware for MCP endpoints.

mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use routes_app::middleware::access_request_cache_needle;
use serde_json::Value;
use services::{
  test_utils::TEST_TENANT_ID, AppAccessRequestStatus, AppService, ApprovedResources,
  ApprovedResourcesV1, McpGrant, McpRequest, McpServerRequest, ModelGrant,
};
use std::sync::Arc;
use utils::{start_test_live_server, ExternalTokenSimulator, EXTERNAL_USER_ID};

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

// =============================================================================
// Guarded route: GET /bodhi/v1/apps/mcps/{id} (access_request_auth_middleware)
// =============================================================================

/// Seeds an MCP server + one instance owned by the external user under
/// TEST_TENANT_ID; returns the instance id for grant lists and show requests.
async fn seed_mcp_instance(app_service: &Arc<dyn AppService>) -> anyhow::Result<String> {
  seed_mcp_instance_in_tenant(app_service, TEST_TENANT_ID).await
}

async fn seed_mcp_instance_in_tenant(
  app_service: &Arc<dyn AppService>,
  tenant_id: &str,
) -> anyhow::Result<String> {
  let mcp_service = app_service.mcp_service();
  let mcp_server = mcp_service
    .create_mcp_server(
      tenant_id,
      EXTERNAL_USER_ID,
      McpServerRequest {
        url: "https://mcp.guard-test.example.com/mcp".to_string(),
        name: "Guard Test Server".to_string(),
        description: None,
        enabled: true,
        auth_config: None,
      },
    )
    .await?;
  let mcp = mcp_service
    .create(
      tenant_id,
      EXTERNAL_USER_ID,
      McpRequest {
        name: "Guarded MCP".to_string(),
        slug: "guarded".to_string(),
        mcp_server_id: Some(mcp_server.id.clone()),
        description: None,
        enabled: true,
        auth_type: Default::default(),
        auth_config_id: None,
        credentials: None,
        oauth_token_id: None,
      },
    )
    .await?;
  Ok(mcp.id)
}

fn mcp_grants(mcp_id: &str) -> ApprovedResources {
  ApprovedResources::V1(ApprovedResourcesV1 {
    models_list: false,
    models_access: ModelGrant::Specific { ids: vec![] },
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific {
      ids: vec![mcp_id.to_string()],
    },
  })
}

async fn get_apps_mcp_show(
  base_url: &str,
  mcp_id: &str,
  bearer_token: &str,
) -> anyhow::Result<reqwest::Response> {
  let client = reqwest::Client::new();
  Ok(
    client
      .get(format!("{}/bodhi/v1/apps/mcps/{}", base_url, mcp_id))
      .header("Authorization", format!("Bearer {}", bearer_token))
      .send()
      .await?,
  )
}

#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_backing_row_passes_guarded_route() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let mcp_id = seed_mcp_instance(&server.app_service).await?;

  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let (bearer_token, _row_id) = simulator
    .create_token_with_backing_row(
      Some("scope_user_user"),
      "test-external-app",
      Some(mcp_grants(&mcp_id)),
    )
    .await?;

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::OK,
    response.status(),
    "Approved backing row matching the token must pass the per-request guard"
  );
  let body: Value = response.json().await?;
  assert_eq!(mcp_id, body["id"].as_str().expect("mcp id present"));

  server.handle.shutdown().await?;
  Ok(())
}

#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_revoked_row_rejected_on_guarded_route() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let mcp_id = seed_mcp_instance(&server.app_service).await?;

  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let grants = mcp_grants(&mcp_id);
  let row = services::AppAccessRequest {
    status: AppAccessRequestStatus::Revoked,
    ..simulator.approved_row("test-external-app", &Some(grants.clone()))?
  };
  let (bearer_token, _row_id) = simulator
    .create_token_for_row(
      Some("scope_user_user"),
      "test-external-app",
      Some(grants),
      row,
    )
    .await?;

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "Revoked access request must be rejected by the per-request guard"
  );
  let body: Value = response.json().await?;
  assert_eq!(
    "access_request_auth_error-access_request_not_approved",
    body["error"]["code"].as_str().expect("error code present")
  );

  server.handle.shutdown().await?;
  Ok(())
}

#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_mismatched_app_client_rejected() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let mcp_id = seed_mcp_instance(&server.app_service).await?;

  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let grants = mcp_grants(&mcp_id);
  // Row approved for a DIFFERENT app than the token's azp.
  let row = services::AppAccessRequest {
    app_client_id: "some-other-app".to_string(),
    ..simulator.approved_row("test-external-app", &Some(grants.clone()))?
  };
  let (bearer_token, _row_id) = simulator
    .create_token_for_row(
      Some("scope_user_user"),
      "test-external-app",
      Some(grants),
      row,
    )
    .await?;

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "Access request approved for a different app must be rejected"
  );
  let body: Value = response.json().await?;
  assert_eq!(
    "access_request_auth_error-app_client_mismatch",
    body["error"]["code"].as_str().expect("error code present")
  );

  server.handle.shutdown().await?;
  Ok(())
}

#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_oauth_token_with_mismatched_user_rejected() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let mcp_id = seed_mcp_instance(&server.app_service).await?;

  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let grants = mcp_grants(&mcp_id);
  // Row approved by a DIFFERENT user than the token's sub.
  let row = services::AppAccessRequest {
    user_id: Some("someone-else".to_string()),
    ..simulator.approved_row("test-external-app", &Some(grants.clone()))?
  };
  let (bearer_token, _row_id) = simulator
    .create_token_for_row(
      Some("scope_user_user"),
      "test-external-app",
      Some(grants),
      row,
    )
    .await?;

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "Access request approved by a different user must be rejected"
  );
  let body: Value = response.json().await?;
  assert_eq!(
    "access_request_auth_error-user_mismatch",
    body["error"]["code"].as_str().expect("error code present")
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// Revocation takes effect immediately on the guarded route.
///
/// Replicates the revoke handler's two effects directly (DB `update_revocation`
/// + evicting cached exchanges via `access_request_cache_needle`) rather than
/// driving POST /access-requests/{id}/revoke: the session-authenticated endpoint
/// only revokes rows owned by the session user, but this row belongs to the
/// simulator's external user, so the endpoint could never find it.
///
/// Uses the REAL tenant (not TEST_TENANT_ID) and a full-claims bearer, so the
/// post-eviction request re-enters `handle_external_client_token` and is
/// rejected by the production scope validation (row no longer Approved).
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_revoked_access_request_rejected_immediately() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let tenant = server
    .app_service
    .tenant_service()
    .get_standalone_app()
    .await?
    .expect("standalone tenant must exist");

  let mcp_id = seed_mcp_instance_in_tenant(&server.app_service, &tenant.id).await?;

  let simulator = ExternalTokenSimulator::new(&server.app_service);
  let (bearer_token, row_id) = simulator
    .create_revalidatable_token(
      Some("scope_user_user"),
      "test-external-app",
      Some(mcp_grants(&mcp_id)),
      &tenant.id,
      &tenant.client_id,
    )
    .await?;

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::OK,
    response.status(),
    "Guarded route must be accessible before revocation"
  );

  server
    .app_service
    .db_service()
    .update_revocation(&tenant.id, &row_id, EXTERNAL_USER_ID)
    .await?;
  server
    .app_service
    .cache_service()
    .remove_entries_containing(&access_request_cache_needle(&row_id));

  let response = get_apps_mcp_show(&server.base_url, &mcp_id, &bearer_token).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "Same request must be rejected immediately after revocation"
  );
  let body: Value = response.json().await?;
  assert_eq!(
    "access_request_validation_error-not_approved",
    body["error"]["code"].as_str().expect("error code present")
  );

  server.handle.shutdown().await?;
  Ok(())
}
