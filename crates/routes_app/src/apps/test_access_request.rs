use crate::test_utils::RequestAuthContextExt;
use crate::{
  apps::AccessRequestActionResponse, apps_approve_access_request, apps_deny_access_request,
  ENDPOINT_ACCESS_REQUESTS_APPROVE, ENDPOINT_ACCESS_REQUESTS_DENY,
};
use anyhow_trace::anyhow_trace;
use axum::{body::Body, http::StatusCode, routing::put};
use axum::{routing::post, Router};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use services::AppAccessRequestStatus;
use services::AuthContext;
use services::ResourceRole;
use services::{
  test_utils::{AppServiceStubBuilder, FrozenTimeService, TEST_TENANT_ID},
  DbService, DefaultAccessRequestService, MockAuthService, RegisterAccessRequestConsentResponse,
  {AppAccessRequest, FlowType},
};
use std::sync::Arc;
use tower::ServiceExt;

// ============================================================================
// Helper: build AppServiceStub with real DB, real AccessRequestService
// ============================================================================

struct TestHarness {
  state: Arc<dyn services::AppService>,
  db_service: Arc<dyn DbService>,
}

async fn build_test_harness(mock_auth: MockAuthService) -> anyhow::Result<TestHarness> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_db_service().await.with_session_service().await;
  let db_service = builder.get_db_service().await;
  let time_service: Arc<dyn services::TimeService> = Arc::new(FrozenTimeService::default());
  let auth_service: Arc<dyn services::AuthService> = Arc::new(mock_auth);

  // Real AccessRequestService backed by same DB + mock auth
  builder.with_tenant(services::Tenant::test_default()).await;
  let access_request_service: Arc<dyn services::AccessRequestService> =
    Arc::new(DefaultAccessRequestService::new(
      db_service.clone(),
      auth_service.clone(),
      time_service.clone(),
      "http://localhost:1135".to_string(),
    ));

  let app_service = builder
    .auth_service(auth_service)
    .access_request_service(access_request_service)
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  Ok(TestHarness { state, db_service })
}

// ============================================================================
// Helper: seed a draft access request in DB
// ============================================================================

async fn seed_draft_request(
  db_service: &dyn DbService,
  request_id: &str,
  flow_type: FlowType,
  redirect_uri: Option<&str>,
  requested_role: &str,
) -> anyhow::Result<AppAccessRequest> {
  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: request_id.to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "test-app-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type,
    redirect_uri: redirect_uri.map(|u| u.to_string()),
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"version":"1","mcp_servers":[{"url":"https://mcp.example.com/mcp"}]}"#
      .to_string(),
    approved: None,
    user_id: None,
    requested_role: requested_role.to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: now + chrono::Duration::seconds(600),
    created_at: now,
    updated_at: now,
  };
  let result = db_service.create(&row).await?;
  Ok(result)
}

// ============================================================================
// Helper: seed an MCP server + user instance, returns the instance id
// ============================================================================

async fn seed_mcp_instance(
  app_service: &dyn services::AppService,
  user_id: &str,
  server_url: &str,
  slug: &str,
  enabled: bool,
) -> anyhow::Result<String> {
  let mcp_service = app_service.mcp_service();
  let server = mcp_service
    .create_mcp_server(
      TEST_TENANT_ID,
      user_id,
      services::McpServerRequest {
        url: server_url.to_string(),
        name: format!("Server {slug}"),
        description: None,
        enabled: true,
        auth_config: None,
      },
    )
    .await?;
  let instance = mcp_service
    .create(
      TEST_TENANT_ID,
      user_id,
      services::McpRequest {
        name: format!("Instance {slug}"),
        slug: slug.to_string(),
        mcp_server_id: Some(server.id.clone()),
        description: None,
        enabled,
        auth_type: services::McpAuthType::Public,
        auth_config_id: None,
        credentials: None,
        oauth_token_id: None,
      },
    )
    .await?;
  Ok(instance.id)
}

// ============================================================================
// apps_approve_access_request - success
// ============================================================================

#[rstest]
#[case::popup_flow(FlowType::Popup, None, false)]
#[case::redirect_flow(FlowType::Redirect, Some("https://app.com/cb"), true)]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_success(
  #[case] flow_type: FlowType,
  #[case] redirect_url: Option<&str>,
  #[case] expect_redirect: bool,
) -> anyhow::Result<()> {
  let request_id = "ar-approve-ok";
  let user_id = "test-user-1";

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_register_access_request_consent()
    .times(1)
    .returning(move |_, _, _, _| {
      Ok(RegisterAccessRequestConsentResponse {
        access_request_id: "ar-approve-ok".to_string(),
        access_request_scope: "scope_access_request:ar-approve-ok".to_string(),
      })
    });

  let harness = build_test_harness(mock_auth).await?;
  let expected_flow_type = flow_type.clone();
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    flow_type,
    redirect_url,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": []
    }
  });

  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::User,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let result = response.json::<AccessRequestActionResponse>().await?;
  assert_eq!(AppAccessRequestStatus::Approved, result.status);
  assert_eq!(expected_flow_type, result.flow_type);
  if expect_redirect {
    assert!(
      result.redirect_url.is_some(),
      "redirect_url should be present for redirect flow"
    );
  } else {
    assert!(
      result.redirect_url.is_none(),
      "redirect_url should be absent for popup flow"
    );
  }

  Ok(())
}

// ============================================================================
// apps_approve_access_request - error: MCP instance not owned
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_mcp_instance_not_owned() -> anyhow::Result<()> {
  let request_id = "ar-not-owned";
  let user_id = "test-user-2";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;
  // No MCP instance seeded for this user -> "not owned"

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": [{
        "url": "https://mcp.example.com/mcp",
        "status": "approved",
        "instance": {"id": "nonexistent-instance"}
      }]
    }
  });

  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::User,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-mcp_instance_not_owned",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// apps_approve_access_request - cross-URL instance (no URL-match restriction)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_cross_url_instance() -> anyhow::Result<()> {
  let request_id = "ar-cross-url";
  let user_id = "test-user-cross";

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_register_access_request_consent()
    .times(1)
    .returning(move |_, _, _, _| {
      Ok(RegisterAccessRequestConsentResponse {
        access_request_id: "ar-cross-url".to_string(),
        access_request_scope: "scope_access_request:ar-cross-url".to_string(),
      })
    });

  let harness = build_test_harness(mock_auth).await?;
  // Requested URL is https://mcp.example.com/mcp (from seed_draft_request); the user's only
  // instance points at a different gateway URL. Approval must still succeed.
  let instance_id = seed_mcp_instance(
    harness.state.as_ref(),
    user_id,
    "https://gateway.composio.dev/gmail/mcp",
    "gmail-via-gateway",
    true,
  )
  .await?;

  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": [{
        "url": "https://mcp.example.com/mcp",
        "status": "approved",
        "instance": {"id": instance_id}
      }]
    }
  });

  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::User,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let result = response.json::<AccessRequestActionResponse>().await?;
  assert_eq!(AppAccessRequestStatus::Approved, result.status);

  Ok(())
}

// ============================================================================
// apps_get_access_request_review - lists all instances, exact-URL match first
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_review_lists_all_instances_match_first() -> anyhow::Result<()> {
  let request_id = "ar-review-order";
  let user_id = "test-user-review";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;

  // One instance on a different URL, one on the exact requested URL.
  seed_mcp_instance(
    harness.state.as_ref(),
    user_id,
    "https://gateway.composio.dev/gmail/mcp",
    "gmail-gateway",
    true,
  )
  .await?;
  let matching_id = seed_mcp_instance(
    harness.state.as_ref(),
    user_id,
    "https://mcp.example.com/mcp",
    "gmail-direct",
    true,
  )
  .await?;

  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      crate::ENDPOINT_ACCESS_REQUESTS_REVIEW,
      axum::routing::get(crate::apps_get_access_request_review),
    )
    .with_state(harness.state);

  let request = axum::http::Request::builder()
    .method("GET")
    .uri(format!("/bodhi/v1/access-requests/{}/review", request_id))
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session(
      user_id,
      "user@test.com",
      ResourceRole::User,
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  let instances = body["mcps_info"][0]["instances"]
    .as_array()
    .expect("instances array");
  assert_eq!(2, instances.len(), "both configured instances are listed");
  // Exact-URL match is sorted first.
  assert_eq!(matching_id, instances[0]["id"].as_str().unwrap());
  assert_eq!(
    "https://mcp.example.com/mcp",
    instances[0]["mcp_server"]["url"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// apps_deny_access_request - success
// ============================================================================

#[rstest]
#[case::popup_flow(FlowType::Popup, None, false)]
#[case::redirect_flow(FlowType::Redirect, Some("https://app.com/cb"), true)]
#[tokio::test]
#[anyhow_trace]
async fn test_deny_access_request_success(
  #[case] flow_type: FlowType,
  #[case] redirect_url: Option<&str>,
  #[case] expect_redirect: bool,
) -> anyhow::Result<()> {
  let expected_flow_type = flow_type.clone();
  let request_id = &format!("ar-deny-{}", flow_type);
  let user_id = "test-user-5";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    flow_type,
    redirect_url,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_DENY,
      post(apps_deny_access_request),
    )
    .with_state(harness.state);

  let request = axum::http::Request::builder()
    .method("POST")
    .uri(format!("/bodhi/v1/access-requests/{}/deny", request_id))
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session(
      user_id,
      "user@test.com",
      ResourceRole::User,
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let result = response.json::<AccessRequestActionResponse>().await?;
  assert_eq!(AppAccessRequestStatus::Denied, result.status);
  assert_eq!(expected_flow_type, result.flow_type);
  if expect_redirect {
    assert!(
      result.redirect_url.is_some(),
      "redirect_url should be present for redirect flow"
    );
  } else {
    assert!(
      result.redirect_url.is_none(),
      "redirect_url should be absent for popup flow"
    );
  }

  Ok(())
}

// ============================================================================
// apps_approve_access_request - error: privilege escalation (user grants power_user)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_privilege_escalation_user_grants_power_user() -> anyhow::Result<()> {
  let request_id = "ar-priv-esc";
  let user_id = "test-user-6";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  // Request asks for power_user scope
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_power_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_power_user",
    "approved": {
      "version": "1",
      "mcps": []
    }
  });

  // User has resource_user role — max grantable is scope_user_user, cannot grant scope_user_power_user
  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::User,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-privilege_escalation",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// apps_approve_access_request - success: power_user downgrades scope_user_power_user to scope_user_user
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_valid_downgrade_power_user_grants_user() -> anyhow::Result<()> {
  let request_id = "ar-downgrade";
  let user_id = "test-user-7";

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_register_access_request_consent()
    .times(1)
    .returning(move |_, _, _, _| {
      Ok(RegisterAccessRequestConsentResponse {
        access_request_id: "ar-downgrade".to_string(),
        access_request_scope: "scope_access_request:ar-downgrade".to_string(),
      })
    });

  let harness = build_test_harness(mock_auth).await?;
  // Request asks for power_user scope
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_power_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": []
    }
  });

  // User has resource_power_user role — max grantable is scope_user_power_user
  // Approving scope_user_user (downgrade) is valid
  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::PowerUser,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let result = response.json::<AccessRequestActionResponse>().await?;
  assert_eq!(AppAccessRequestStatus::Approved, result.status);

  Ok(())
}

// ============================================================================
// apps_approve_access_request - error: approved_role exceeds requested_role
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_privilege_escalation_approved_exceeds_requested() -> anyhow::Result<()> {
  let request_id = "ar-priv-esc-exceed";
  let user_id = "test-user-8";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  // Request only asks for scope_user_user
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_power_user",
    "approved": {
      "version": "1",
      "mcps": []
    }
  });

  // Approver has PowerUser role (can grant up to power_user), but approved_role exceeds requested_role
  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::PowerUser,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-privilege_escalation",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// Version validation tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_access_request_unknown_version() -> anyhow::Result<()> {
  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;

  let router = Router::new()
    .route(
      crate::ENDPOINT_APPS_REQUEST_ACCESS,
      post(crate::apps_create_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "app_client_id": "test-app-client",
    "flow_type": "popup",
    "requested_role": "scope_user_user",
    "requested": {
      "version": "99",
      "mcp_servers": []
    }
  });

  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/apps/request-access")
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session(
      "user-1",
      "user@test.com",
      ResourceRole::User,
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "json_rejection_error",
    body["error"]["code"].as_str().unwrap()
  );
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("Unsupported resources version"),
    "Expected message about unsupported version, got: {}",
    message
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_unknown_version() -> anyhow::Result<()> {
  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  let request_id = "ar-unknown-ver";
  let user_id = "user-1";

  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "99",
      "mcps": []
    }
  });

  let request = axum::http::Request::builder()
    .method("PUT")
    .uri(format!("/bodhi/v1/access-requests/{}/approve", request_id))
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_session_with_token(
      user_id,
      "user@test.com",
      ResourceRole::User,
      "dummy-token",
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "json_rejection_error",
    body["error"]["code"].as_str().unwrap()
  );
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("Unsupported resources version"),
    "Expected message about unsupported version, got: {}",
    message
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_review_access_request_invalid_requested_json() -> anyhow::Result<()> {
  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;

  // Seed a row with invalid JSON in the requested field
  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: "ar-bad-json".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "test-app-client".to_string(),
    app_name: None,
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: "not-valid-json".to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: now + chrono::Duration::seconds(600),
    created_at: now,
    updated_at: now,
  };
  harness.db_service.create(&row).await?;

  let router = Router::new()
    .route(
      crate::ENDPOINT_ACCESS_REQUESTS_REVIEW,
      axum::routing::get(crate::apps_get_access_request_review),
    )
    .with_state(harness.state);

  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/access-requests/ar-bad-json/review")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session(
      "user-1",
      "user@test.com",
      ResourceRole::User,
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-invalid_requested_json",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}
