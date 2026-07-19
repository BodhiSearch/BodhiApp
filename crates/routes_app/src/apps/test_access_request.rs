use crate::test_utils::RequestAuthContextExt;
use crate::{
  apps::{
    AccessRequestActionResponse, AppAccessSummary, CreateAccessRequestResponse,
    ListAppAccessResponse,
  },
  apps_approve_access_request, apps_create_access_request, apps_deny_access_request,
  apps_list_user_access, apps_revoke_access_request, ResourceAccess,
  ENDPOINT_ACCESS_REQUESTS_APPROVE, ENDPOINT_ACCESS_REQUESTS_APPS, ENDPOINT_ACCESS_REQUESTS_DENY,
  ENDPOINT_ACCESS_REQUESTS_REVOKE, ENDPOINT_APPS_REQUEST_ACCESS,
};
use anyhow_trace::anyhow_trace;
use axum::{body::Body, http::StatusCode, routing::put};
use axum::{
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use services::AppAccessRequestStatus;
use services::AuthContext;
use services::ResourceRole;
use services::{
  test_utils::{
    AppServiceStubBuilder, FrozenTimeService, SettingServiceStub, StubNetworkService,
    TEST_TENANT_ID,
  },
  AppAccessRequest, DbService, DefaultAccessRequestService, MockAuthService,
  RegisterAccessRequestConsentResponse,
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

  builder.with_tenant(services::Tenant::test_default()).await;
  let access_request_service: Arc<dyn services::AccessRequestService> =
    Arc::new(DefaultAccessRequestService::new(
      db_service.clone(),
      auth_service.clone(),
      time_service.clone(),
      Arc::new(SettingServiceStub::default()),
      Arc::new(StubNetworkService { ip: None }),
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
  requested_role: &str,
) -> anyhow::Result<AppAccessRequest> {
  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: request_id.to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "test-app-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"version":"1","mcp_servers":[{"url":"https://mcp.example.com/mcp"}]}"#
      .to_string(),
    approved: None,
    user_id: None,
    requested_role: requested_role.to_string(),
    approved_role: None,
    access_request_scope: None,
    source_access_request_id: None,
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
// Helper: seed an APPROVED access request owned by `user_id`
// ============================================================================

#[allow(clippy::too_many_arguments)]
async fn seed_approved_request(
  db_service: &dyn DbService,
  request_id: &str,
  user_id: &str,
  app_client_id: &str,
  approved_json: &str,
) -> anyhow::Result<AppAccessRequest> {
  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: request_id.to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: app_client_id.to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(approved_json.to_string()),
    user_id: Some(user_id.to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(format!("scope_access_request:{request_id}")),
    source_access_request_id: None,
    error_message: None,
    expires_at: now + chrono::Duration::seconds(600),
    created_at: now,
    updated_at: now,
  };
  let result = db_service.create(&row).await?;
  Ok(result)
}

fn management_router(state: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(ENDPOINT_ACCESS_REQUESTS_APPS, get(apps_list_user_access))
    .route(
      ENDPOINT_ACCESS_REQUESTS_REVOKE,
      post(apps_revoke_access_request),
    )
    .with_state(state)
}

// ============================================================================
// apps_list_user_access — list issued app tokens with grant summary
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_user_access_returns_only_callers_approved_with_summary() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  seed_approved_request(
    harness.db_service.as_ref(),
    "ar-mine",
    "owner-1",
    "app-mine",
    r#"{"version":"1","models_list":true,"models_access":{"type":"specific","ids":["alias-x"]},"mcps_list":false,"mcps_access":{"type":"specific","ids":["mcp-1"]}}"#,
  )
  .await?;
  // Another user's grant must NOT appear.
  seed_approved_request(
    harness.db_service.as_ref(),
    "ar-other",
    "owner-2",
    "app-other",
    r#"{"version":"1"}"#,
  )
  .await?;

  let router = management_router(harness.state);
  let request = axum::http::Request::builder()
    .method("GET")
    .uri(ENDPOINT_ACCESS_REQUESTS_APPS)
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "owner-1",
      "owner1@test.com",
      ResourceRole::User,
      "dummy-token",
    ));
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<ListAppAccessResponse>().await?;
  assert_eq!(1, body.data.len());
  let summary = &body.data[0];
  assert_eq!("ar-mine", summary.id);
  assert_eq!("app-mine", summary.app_client_id);
  assert_eq!(
    ResourceAccess::Specific {
      list: true,
      ids: vec!["alias-x".to_string()]
    },
    summary.models
  );
  assert_eq!(
    ResourceAccess::Specific {
      list: false,
      ids: vec!["mcp-1".to_string()]
    },
    summary.mcps
  );
  Ok(())
}

// ============================================================================
// apps_revoke_access_request — revoke makes the grant inactive
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_revoke_access_request_transitions_to_revoked() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  seed_approved_request(
    harness.db_service.as_ref(),
    "ar-revoke",
    "owner-1",
    "app-1",
    r#"{"version":"1"}"#,
  )
  .await?;

  let router = management_router(harness.state.clone());
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-revoke/revoke")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "owner-1",
      "owner1@test.com",
      ResourceRole::User,
      "dummy-token",
    ));
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  let summary = response.json::<AppAccessSummary>().await?;
  assert_eq!(AppAccessRequestStatus::Revoked, summary.status);

  // After revoke it no longer appears in the caller's active list.
  let list_router = management_router(harness.state);
  let list_req = axum::http::Request::builder()
    .method("GET")
    .uri(ENDPOINT_ACCESS_REQUESTS_APPS)
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "owner-1",
      "owner1@test.com",
      ResourceRole::User,
      "dummy-token",
    ));
  let list_resp = list_router.oneshot(list_req).await?;
  let body = list_resp.json::<ListAppAccessResponse>().await?;
  assert_eq!(0, body.data.len());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_revoke_access_request_not_owner_rejected() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  seed_approved_request(
    harness.db_service.as_ref(),
    "ar-revoke-2",
    "owner-1",
    "app-1",
    r#"{"version":"1"}"#,
  )
  .await?;

  let router = management_router(harness.state);
  // A different user attempts the revoke.
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-revoke-2/revoke")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "attacker",
      "attacker@test.com",
      ResourceRole::User,
      "dummy-token",
    ));
  let response = router.oneshot(request).await?;
  assert_ne!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_revoke_non_approved_request_rejected() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  // A draft (never approved) cannot be revoked.
  seed_draft_request(
    harness.db_service.as_ref(),
    "ar-draft-revoke",
    "scope_user_user",
  )
  .await?;

  let router = management_router(harness.state);
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-draft-revoke/revoke")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "owner-1",
      "owner1@test.com",
      ResourceRole::User,
      "dummy-token",
    ));
  let response = router.oneshot(request).await?;
  assert_ne!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// apps_approve_access_request - success
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_success() -> anyhow::Result<()> {
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
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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
  assert_eq!(
    Some("scope_access_request:ar-approve-ok".to_string()),
    result.access_request_scope
  );

  Ok(())
}

// ============================================================================
// apps_create_access_request - review_url reflects the request Host header
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_access_request_review_url_reflects_host() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  let router = Router::new()
    .route(
      ENDPOINT_APPS_REQUEST_ACCESS,
      post(apps_create_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "app_client_id": "app-client-1",
    "requested_role": "scope_user_user",
    "requested": { "version": "1", "mcp_servers": [] }
  });

  // A loopback Host is reflected into the review URL (fixes the default 0.0.0.0 link).
  let request = axum::http::Request::builder()
    .method("POST")
    .uri(ENDPOINT_APPS_REQUEST_ACCESS)
    .header("Content-Type", "application/json")
    .header("Host", "127.0.0.1:1135")
    .body(Body::from(serde_json::to_string(&body)?))?;
  let response = router.clone().oneshot(request).await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<CreateAccessRequestResponse>().await?;
  assert_eq!(
    format!(
      "http://127.0.0.1:1135/ui/apps/access-requests/review?id={}",
      created.id
    ),
    created.review_url
  );

  // No Host header → falls back to the configured public server URL (localhost default).
  let request = axum::http::Request::builder()
    .method("POST")
    .uri(ENDPOINT_APPS_REQUEST_ACCESS)
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?;
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<CreateAccessRequestResponse>().await?;
  assert_eq!(
    format!(
      "http://localhost:1135/ui/apps/access-requests/review?id={}",
      created.id
    ),
    created.review_url
  );

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
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;
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
// apps_approve_access_request - extended grants (models + owner-extra MCPs)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_with_model_and_extra_mcp_grants() -> anyhow::Result<()> {
  let request_id = "ar-extended";
  let user_id = "test-user-extended";

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_register_access_request_consent()
    .times(1)
    .returning(move |_, _, _, _| {
      Ok(RegisterAccessRequestConsentResponse {
        access_request_id: "ar-extended".to_string(),
        access_request_scope: "scope_access_request:ar-extended".to_string(),
      })
    });

  let harness = build_test_harness(mock_auth).await?;
  // Owner-extra MCP must reference an owned + enabled instance.
  let extra_id = seed_mcp_instance(
    harness.state.as_ref(),
    user_id,
    "https://extra.example.com/mcp",
    "extra-tool",
    true,
  )
  .await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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
      "models_list": true,
      "models_access": {"type": "specific", "ids": ["alias-x"]},
      "mcps_list": false,
      "mcps": [],
      "mcps_access": {"type": "specific", "ids": [extra_id]}
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_extra_mcp_not_owned() -> anyhow::Result<()> {
  let request_id = "ar-extra-not-owned";
  let user_id = "test-user-extra-no";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .with_state(harness.state);

  // Owner-extra grant references an instance the user does not own → 403.
  let body = json!({
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": [],
      "mcps_access": {"type": "specific", "ids": ["nonexistent-extra"]}
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

  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_authorize_url()
    .return_const("https://kc.example.com/realms/bodhi/protocol/openid-connect/auth".to_string());
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

  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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
  assert_eq!(matching_id, instances[0]["id"].as_str().unwrap());
  assert_eq!(
    "https://mcp.example.com/mcp",
    instances[0]["mcp_server"]["url"].as_str().unwrap()
  );
  assert_eq!(
    "https://kc.example.com/realms/bodhi/protocol/openid-connect/auth",
    body["auth_endpoint"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// apps_deny_access_request - success
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_deny_access_request_success() -> anyhow::Result<()> {
  let request_id = "ar-deny-1";
  let user_id = "test-user-5";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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
  assert_eq!(None, result.access_request_scope);

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
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
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
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
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
  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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

  seed_draft_request(harness.db_service.as_ref(), request_id, "scope_user_user").await?;

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

  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: "ar-bad-json".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "test-app-client".to_string(),
    app_name: None,
    app_description: None,
    status: AppAccessRequestStatus::Draft,
    requested: "not-valid-json".to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    source_access_request_id: None,
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

#[test]
fn app_access_summary_clamps_tampered_approved_role() {
  use services::UserScope;
  let ts: chrono::DateTime<chrono::Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
  let row = |approved_role: Option<&str>| AppAccessRequest {
    id: "ar-1".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "app".to_string(),
    app_name: None,
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: None,
    user_id: Some("u".to_string()),
    requested_role: "scope_user_power_user".to_string(),
    approved_role: approved_role.map(|s| s.to_string()),
    access_request_scope: None,
    source_access_request_id: None,
    error_message: None,
    expires_at: ts,
    created_at: ts,
    updated_at: ts,
  };

  // A (DB-tampered) role above the caller's ceiling is clamped down for display.
  let s =
    crate::AppAccessSummary::from_row(row(Some("scope_user_power_user")), Some(UserScope::User));
  assert_eq!(Some(UserScope::User), s.approved_role);
  // Within the ceiling ⇒ unchanged.
  let s =
    crate::AppAccessSummary::from_row(row(Some("scope_user_user")), Some(UserScope::PowerUser));
  assert_eq!(Some(UserScope::User), s.approved_role);
  // No ceiling (non-session principal) ⇒ no clamp.
  let s = crate::AppAccessSummary::from_row(row(Some("scope_user_power_user")), None);
  assert_eq!(Some(UserScope::PowerUser), s.approved_role);
}

// ============================================================================
// apps_create_access_request - exchange / upgrade mode
// ============================================================================

/// Seed a Draft whose `source_access_request_id` points at a prior request.
async fn seed_draft_with_source(
  db_service: &dyn DbService,
  request_id: &str,
  app_client_id: &str,
  source_id: Option<&str>,
) -> anyhow::Result<AppAccessRequest> {
  let now = chrono::Utc::now();
  let row = AppAccessRequest {
    id: request_id.to_string(),
    tenant_id: None,
    app_client_id: app_client_id.to_string(),
    app_name: Some("Upgrade App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"version":"1","models_access":true,"mcp_servers":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    source_access_request_id: source_id.map(|s| s.to_string()),
    error_message: None,
    expires_at: now + chrono::Duration::seconds(600),
    created_at: now,
    updated_at: now,
  };
  Ok(db_service.create(&row).await?)
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_access_request_exchange_stores_source_id() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  let router = Router::new()
    .route(
      crate::ENDPOINT_APPS_REQUEST_ACCESS,
      post(crate::apps_create_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "app_client_id": "test-app-client",
    "requested_role": "scope_user_user",
    "requested": {"version": "1", "mcp_servers": []},
    "exchange": true
  });
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/apps/request-access")
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_external_app(
      "user-1",
      services::UserScope::User,
      "test-app-client",
      Some("ar-source-1"),
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<Value>().await?;
  let created_id = created["id"].as_str().unwrap().to_string();

  let draft = harness
    .db_service
    .get("", &created_id)
    .await?
    .expect("draft persisted");
  assert_eq!(
    Some("ar-source-1".to_string()),
    draft.source_access_request_id
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_access_request_exchange_without_auth_rejected() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  let router = Router::new()
    .route(
      crate::ENDPOINT_APPS_REQUEST_ACCESS,
      post(crate::apps_create_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "app_client_id": "test-app-client",
    "requested_role": "scope_user_user",
    "requested": {"version": "1", "mcp_servers": []},
    "exchange": true
  });
  // No auth context populated (anonymous) — exchange must be rejected.
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/apps/request-access")
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-exchange_requires_auth",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_access_request_exchange_app_client_mismatch_rejected() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  let router = Router::new()
    .route(
      crate::ENDPOINT_APPS_REQUEST_ACCESS,
      post(crate::apps_create_access_request),
    )
    .with_state(harness.state);

  let body = json!({
    "app_client_id": "test-app-client",
    "requested_role": "scope_user_user",
    "requested": {"version": "1", "mcp_servers": []},
    "exchange": true
  });
  // Token belongs to a different app than the one named in the request body.
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/apps/request-access")
    .header("Content-Type", "application/json")
    .body(Body::from(serde_json::to_string(&body)?))?
    .with_auth_context(AuthContext::test_external_app(
      "user-1",
      services::UserScope::User,
      "other-app-client",
      Some("ar-source-1"),
    ));

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

// ============================================================================
// apps_get_access_request_review - previous_grant embedding (upgrade)
// ============================================================================

fn review_router(state: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(
      crate::ENDPOINT_ACCESS_REQUESTS_REVIEW,
      get(crate::apps_get_access_request_review),
    )
    .with_state(state)
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_review_embeds_previous_grant_for_exchange() -> anyhow::Result<()> {
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_authorize_url()
    .return_const("https://kc.example.com/auth".to_string());
  let harness = build_test_harness(mock_auth).await?;

  let approved_json = r#"{"version":"1","models_list":true,"models_access":{"type":"specific","ids":["model-a"]},"mcps_list":true,"mcps":[],"mcps_access":{"type":"specific","ids":[]}}"#;
  seed_approved_request(
    harness.db_service.as_ref(),
    "ar-source-1",
    "user-1",
    "test-app-client",
    approved_json,
  )
  .await?;
  seed_draft_with_source(
    harness.db_service.as_ref(),
    "ar-upgrade-1",
    "test-app-client",
    Some("ar-source-1"),
  )
  .await?;

  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/access-requests/ar-upgrade-1/review")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session(
      "user-1",
      "user@test.com",
      ResourceRole::User,
    ));

  let response = review_router(harness.state).oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  let prev = &body["previous_grant"];
  assert_eq!("scope_user_user", prev["approved_role"].as_str().unwrap());
  assert_eq!(true, prev["approved"]["models_list"].as_bool().unwrap());
  assert_eq!(
    "model-a",
    prev["approved"]["models_access"]["ids"][0]
      .as_str()
      .unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_review_omits_previous_grant_when_source_missing() -> anyhow::Result<()> {
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_authorize_url()
    .return_const("https://kc.example.com/auth".to_string());
  let harness = build_test_harness(mock_auth).await?;

  seed_draft_with_source(
    harness.db_service.as_ref(),
    "ar-upgrade-2",
    "test-app-client",
    Some("does-not-exist"),
  )
  .await?;

  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/access-requests/ar-upgrade-2/review")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session(
      "user-1",
      "user@test.com",
      ResourceRole::User,
    ));

  let response = review_router(harness.state).oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert!(body["previous_grant"].is_null());
  Ok(())
}
