//! Multi-tenant live integration tests against a real HTTP server and real Keycloak.
//!
//! These tests exercise the full multi-tenant flow: dashboard auth, tenant creation,
//! DAG enablement, resource token acquisition, tenant activation, and info/user endpoints.
//!
//! Requires:
//! - Real Keycloak instance (configured via `tests/resources/.env.test`)
//! - `INTEG_TEST_MULTI_TENANT_CLIENT_ID` and `INTEG_TEST_MULTI_TENANT_CLIENT_SECRET` env vars
//! - Port 51135 available

mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::{json, Value};
use utils::{
  add_resource_token_to_session, create_dashboard_session, get_dashboard_token_via_password_grant,
  get_resource_token_via_password_grant, live_server, start_multitenant_live_server,
  TestServerHandle,
};

/// Helper: read env vars needed for multi-tenant tests.
struct MultiTenantEnv {
  auth_url: String,
  realm: String,
  mt_client_id: String,
  mt_client_secret: String,
  username: String,
  password: String,
}

impl MultiTenantEnv {
  fn load() -> anyhow::Result<Self> {
    Ok(Self {
      auth_url: std::env::var("INTEG_TEST_AUTH_URL")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_URL not set"))?,
      realm: std::env::var("INTEG_TEST_AUTH_REALM")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_REALM not set"))?,
      mt_client_id: std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_ID")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_MULTI_TENANT_CLIENT_ID not set"))?,
      mt_client_secret: std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET not set"))?,
      username: std::env::var("INTEG_TEST_USERNAME")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_USERNAME not set"))?,
      password: std::env::var("INTEG_TEST_PASSWORD")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_PASSWORD not set"))?,
    })
  }

  async fn get_dashboard_token(&self) -> anyhow::Result<String> {
    get_dashboard_token_via_password_grant(
      &self.auth_url,
      &self.realm,
      &self.mt_client_id,
      &self.mt_client_secret,
      &self.username,
      &self.password,
    )
    .await
  }

  async fn get_resource_token(
    &self,
    client_id: &str,
    client_secret: &str,
  ) -> anyhow::Result<String> {
    get_resource_token_via_password_grant(
      &self.auth_url,
      &self.realm,
      client_id,
      client_secret,
      &self.username,
      &self.password,
    )
    .await
  }
}

/// Full end-to-end multi-tenant flow:
///
/// 1. GET /bodhi/v1/info (no cookie) -> tenant_selection, deployment: multi-tenant
/// 2. Get dashboard token via password grant
/// 3. Inject dashboard token into session -> get session cookie
/// 4. DELETE /dev/tenants/cleanup (with cookie) -> clean slate for KC and local DB
/// 5. POST /bodhi/v1/tenants {name, description} -> 201 {client_id}
/// 6. POST /dev/clients/{client_id}/dag (with cookie) -> 200 {client_id, client_secret}
/// 7. Get resource token via password grant (client_id + client_secret + username + password)
/// 8. Update session: add resource token + active_client_id
/// 9. POST /bodhi/v1/tenants/{client_id}/activate (with cookie) -> 200
/// 10. GET /bodhi/v1/info (with cookie) -> ready, client_id matches
/// 11. GET /bodhi/v1/user (with cookie) -> has_dashboard_session: true
/// 12. Cleanup: DELETE /dev/tenants/cleanup (with cookie)
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_multi_tenant_full_flow() -> anyhow::Result<()> {
  let server = start_multitenant_live_server().await?;
  let env = MultiTenantEnv::load()?;
  let client = reqwest::Client::new();

  // Step 1: GET /bodhi/v1/info without session -> tenant_selection
  let resp = client
    .get(format!("{}/bodhi/v1/info", server.base_url))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let info: Value = resp.json().await?;
  assert_eq!("tenant_selection", info["status"].as_str().unwrap());
  assert_eq!("multi_tenant", info["deployment"].as_str().unwrap());

  // Step 2: Get dashboard token
  let dashboard_token = env.get_dashboard_token().await?;

  // Step 3: Inject dashboard token into session
  let (session_cookie, session_id) =
    create_dashboard_session(&server.app_service, &dashboard_token).await?;

  // Step 4: DELETE /dev/tenants/cleanup -> clean slate
  let resp = client
    .delete(format!("{}/dev/tenants/cleanup", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Cleanup failed: {}",
    resp
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );

  // Step 5: POST /bodhi/v1/tenants -> create tenant
  let resp = client
    .post(format!("{}/bodhi/v1/tenants", server.base_url))
    .header("Cookie", &session_cookie)
    .json(&json!({
      "name": "Test Multi-Tenant App",
      "description": "Integration test tenant"
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Create tenant failed: {}",
    resp
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );
  let create_resp: Value = resp.json().await?;
  let new_client_id = create_resp["client_id"]
    .as_str()
    .expect("Expected client_id in response")
    .to_string();

  // Step 6: POST /dev/clients/{client_id}/dag -> enable Direct Access Grants
  let resp = client
    .post(format!(
      "{}/dev/clients/{}/dag",
      server.base_url, new_client_id
    ))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "DAG enable failed: {}",
    resp
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );
  let dag_resp: Value = resp.json().await?;
  let client_secret = dag_resp["client_secret"]
    .as_str()
    .expect("Expected client_secret in DAG response")
    .to_string();
  assert_eq!(new_client_id, dag_resp["client_id"].as_str().unwrap());

  // Step 7: Get resource token via password grant
  let resource_token = env
    .get_resource_token(&new_client_id, &client_secret)
    .await?;

  // Step 8: Update session with resource token + active_client_id
  add_resource_token_to_session(
    &server.app_service,
    session_id,
    &new_client_id,
    &resource_token,
  )
  .await?;

  // Step 9: POST /bodhi/v1/tenants/{client_id}/activate -> 200
  let resp = client
    .post(format!(
      "{}/bodhi/v1/tenants/{}/activate",
      server.base_url, new_client_id
    ))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Activate failed: {}",
    resp
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );

  // Step 10: GET /bodhi/v1/info (with cookie) -> ready, client_id matches
  let resp = client
    .get(format!("{}/bodhi/v1/info", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let info: Value = resp.json().await?;
  assert_eq!("ready", info["status"].as_str().unwrap());
  assert_eq!("multi_tenant", info["deployment"].as_str().unwrap());
  assert_eq!(new_client_id, info["client_id"].as_str().unwrap());

  // Step 11: GET /bodhi/v1/user (with cookie) -> has_dashboard_session: true
  let resp = client
    .get(format!("{}/bodhi/v1/user", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let user_info: Value = resp.json().await?;
  assert_eq!(
    true,
    user_info["has_dashboard_session"]
      .as_bool()
      .unwrap_or(false)
  );

  // Step 12: Cleanup
  // Refresh dashboard token (original may have expired during the test)
  let dashboard_token = env.get_dashboard_token().await?;
  let (cleanup_cookie, _) = create_dashboard_session(&server.app_service, &dashboard_token).await?;
  let resp = client
    .delete(format!("{}/dev/tenants/cleanup", server.base_url))
    .header("Cookie", &cleanup_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  server.handle.shutdown().await?;
  Ok(())
}

/// Info status progression in multi-tenant mode:
///
/// 1. No session -> tenant_selection
/// 2. Dashboard session only, no tenants created -> setup or tenant_selection
/// 3. Dashboard + created & activated tenant -> ready
/// 4. Cleanup
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_info_state_progression() -> anyhow::Result<()> {
  let server = start_multitenant_live_server().await?;
  let env = MultiTenantEnv::load()?;
  let client = reqwest::Client::new();

  // State 1: No session -> tenant_selection
  let resp = client
    .get(format!("{}/bodhi/v1/info", server.base_url))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let info: Value = resp.json().await?;
  assert_eq!("tenant_selection", info["status"].as_str().unwrap());

  // Get dashboard token and create session
  let dashboard_token = env.get_dashboard_token().await?;
  let (session_cookie, session_id) =
    create_dashboard_session(&server.app_service, &dashboard_token).await?;

  // Cleanup any pre-existing tenants
  let resp = client
    .delete(format!("{}/dev/tenants/cleanup", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  // State 2: Dashboard session, no tenants -> setup (empty tenant list)
  // Note: depends on whether the KC SPI returns empty list
  let resp = client
    .get(format!("{}/bodhi/v1/info", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let info: Value = resp.json().await?;
  let status = info["status"].as_str().unwrap();
  assert!(
    status == "setup" || status == "tenant_selection",
    "Expected setup or tenant_selection, got: {}",
    status
  );

  // Create a tenant and activate it
  let resp = client
    .post(format!("{}/bodhi/v1/tenants", server.base_url))
    .header("Cookie", &session_cookie)
    .json(&json!({
      "name": "State Progression Test Tenant",
      "description": "Testing state progression"
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::CREATED, resp.status());
  let create_resp: Value = resp.json().await?;
  let new_client_id = create_resp["client_id"]
    .as_str()
    .expect("Expected client_id in response")
    .to_string();

  // Enable DAG and get resource token
  let resp = client
    .post(format!(
      "{}/dev/clients/{}/dag",
      server.base_url, new_client_id
    ))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let dag_resp: Value = resp.json().await?;
  let client_secret = dag_resp["client_secret"]
    .as_str()
    .expect("Expected client_secret")
    .to_string();

  let resource_token = env
    .get_resource_token(&new_client_id, &client_secret)
    .await?;

  add_resource_token_to_session(
    &server.app_service,
    session_id,
    &new_client_id,
    &resource_token,
  )
  .await?;

  // Activate the tenant
  let resp = client
    .post(format!(
      "{}/bodhi/v1/tenants/{}/activate",
      server.base_url, new_client_id
    ))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  // State 3: Dashboard + activated tenant -> ready
  let resp = client
    .get(format!("{}/bodhi/v1/info", server.base_url))
    .header("Cookie", &session_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let info: Value = resp.json().await?;
  assert_eq!("ready", info["status"].as_str().unwrap());
  assert_eq!(new_client_id, info["client_id"].as_str().unwrap());

  // Cleanup
  let dashboard_token = env.get_dashboard_token().await?;
  let (cleanup_cookie, _) = create_dashboard_session(&server.app_service, &dashboard_token).await?;
  let resp = client
    .delete(format!("{}/dev/tenants/cleanup", server.base_url))
    .header("Cookie", &cleanup_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  server.handle.shutdown().await?;
  Ok(())
}

/// Standalone mode rejects dashboard auth initiate endpoint.
///
/// Uses the existing `live_server` fixture (standalone mode) to verify that
/// POST /bodhi/v1/auth/dashboard/initiate returns NotMultiTenant error.
#[rstest::rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_standalone_rejects_dashboard_auth(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let server = live_server?;
  let base_url = format!("http://{}:{}", server.host, server.port);
  let client = reqwest::Client::new();

  let resp = client
    .post(format!("{}/bodhi/v1/auth/dashboard/initiate", base_url))
    .header("Content-Type", "application/json")
    .body("{}")
    .send()
    .await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());

  let body: Value = resp.json().await?;
  assert_eq!(
    "dashboard_auth_route_error-not_multi_tenant",
    body["error"]["code"].as_str().unwrap()
  );

  server.handle.shutdown().await?;
  Ok(())
}
