use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{test_utils::AppServiceStubBuilder, AppService, MockMcpService};
use std::sync::Arc;

/// Builds a `RouterState` from a `MockMcpService`.
/// Callers build their own `Router` with specific routes and call `.with_state(state)`.
pub async fn build_mcp_test_state(
  mock_mcp_service: MockMcpService,
) -> anyhow::Result<Arc<dyn RouterState>> {
  let mcp_svc: Arc<dyn services::McpService> = Arc::new(mock_mcp_service);
  let app_service = AppServiceStubBuilder::default()
    .mcp_service(mcp_svc)
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  Ok(state)
}

/// Builds a `RouterState` and returns the `AppService` for direct service access in tests.
pub async fn build_mcp_test_state_with_app_service(
  mock_mcp_service: MockMcpService,
) -> anyhow::Result<(Arc<dyn RouterState>, Arc<dyn AppService>)> {
  let mcp_svc: Arc<dyn services::McpService> = Arc::new(mock_mcp_service);
  let app_service: Arc<dyn AppService> = Arc::new(
    AppServiceStubBuilder::default()
      .mcp_service(mcp_svc)
      .build()
      .await?,
  );

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));
  Ok((state, app_service))
}

// ---------------------------------------------------------------------------
// The helpers below are only available when running routes_app's own tests.
// They depend on `tower` (dev-dep) and `objs/test-utils`, both of which are
// only available in test-binary contexts, not when downstream crates enable
// the `test-utils` feature.
// ---------------------------------------------------------------------------

/// Fixed deterministic timestamp matching `FrozenTimeService` default (2025-01-01T00:00:00Z).
#[cfg(test)]
pub use objs::test_utils::fixed_dt;

/// Creates an MCP server in the database via the API and returns its ID.
#[cfg(test)]
pub async fn setup_mcp_server_in_db(
  router: &axum::Router,
  admin_cookie: &str,
) -> anyhow::Result<String> {
  use axum::body::Body;
  use axum::http::StatusCode;
  use serde_json::{json, Value};
  use server_core::test_utils::ResponseTestExt;
  use tower::ServiceExt;

  let body = json!({
    "name": "Test MCP Server",
    "url": "https://mcp.example.com/mcp",
    "description": "Integration test server",
    "enabled": true
  });
  let response = router
    .clone()
    .oneshot(crate::test_utils::session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/servers",
      admin_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let server: Value = response.json().await?;
  Ok(server["id"].as_str().unwrap().to_string())
}

/// Creates a header auth config in the database via the API and returns its ID.
#[cfg(test)]
pub async fn create_header_auth_config_in_db(
  router: &axum::Router,
  user_cookie: &str,
  server_id: &str,
  header_key: &str,
  header_value: &str,
) -> anyhow::Result<String> {
  use axum::body::Body;
  use axum::http::StatusCode;
  use serde_json::{json, Value};
  use server_core::test_utils::ResponseTestExt;
  use tower::ServiceExt;

  let body = json!({
    "mcp_server_id": server_id,
    "type": "header",
    "name": "Header",
    "header_key": header_key,
    "header_value": header_value
  });
  let response = router
    .clone()
    .oneshot(crate::test_utils::session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-configs",
      user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let auth: Value = response.json().await?;
  Ok(auth["id"].as_str().unwrap().to_string())
}

/// Creates an OAuth auth config in the database via the API and returns its ID.
#[cfg(test)]
pub async fn create_oauth_auth_config_in_db(
  router: &axum::Router,
  user_cookie: &str,
  server_id: &str,
  name: &str,
  extra_fields: serde_json::Value,
) -> anyhow::Result<String> {
  use axum::body::Body;
  use axum::http::StatusCode;
  use serde_json::{json, Value};
  use server_core::test_utils::ResponseTestExt;
  use tower::ServiceExt;

  let mut body = json!({
    "mcp_server_id": server_id,
    "type": "oauth",
    "name": name,
  });
  if let (Some(obj), Some(extra_obj)) = (body.as_object_mut(), extra_fields.as_object()) {
    for (k, v) in extra_obj {
      obj.insert(k.clone(), v.clone());
    }
  }
  let response = router
    .clone()
    .oneshot(crate::test_utils::session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-configs",
      user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let auth: Value = response.json().await?;
  Ok(auth["id"].as_str().unwrap().to_string())
}
