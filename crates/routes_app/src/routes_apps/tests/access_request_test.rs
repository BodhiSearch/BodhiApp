use crate::{
  approve_access_request_handler, deny_access_request_handler,
  routes_apps::AccessRequestActionResponse, ENDPOINT_ACCESS_REQUESTS_APPROVE,
  ENDPOINT_ACCESS_REQUESTS_DENY,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{body::Body, http::StatusCode, routing::put};
use axum::{routing::post, Router};
use objs::ResourceRole;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  db::{AppAccessRequestRow, DbService, ToolsetRow},
  test_utils::{AppServiceStubBuilder, FrozenTimeService},
  DefaultAccessRequestService, DefaultToolService, MockAuthService, MockExaService,
  MockSecretService, RegisterAccessRequestConsentResponse,
};
use std::sync::Arc;
use tower::ServiceExt;

// ============================================================================
// Helper: build AppServiceStub with real DB, real ToolService, real AccessRequestService
// ============================================================================

struct TestHarness {
  state: Arc<dyn RouterState>,
  db_service: Arc<dyn DbService>,
}

async fn build_test_harness(mock_auth: MockAuthService) -> anyhow::Result<TestHarness> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_db_service().await.with_session_service().await;
  let db_service = builder.get_db_service().await;
  let time_service: Arc<dyn services::db::TimeService> = Arc::new(FrozenTimeService::default());
  let auth_service: Arc<dyn services::AuthService> = Arc::new(mock_auth);

  // Real ToolService backed by same DB
  let exa = MockExaService::new();
  let tool_service: Arc<dyn services::ToolService> = Arc::new(DefaultToolService::new(
    db_service.clone(),
    Arc::new(exa),
    time_service.clone(),
  ));

  // Real AccessRequestService backed by same DB + mock auth
  let secret_service: Arc<dyn services::SecretService> = Arc::new(MockSecretService::new());
  let access_request_service: Arc<dyn services::AccessRequestService> =
    Arc::new(DefaultAccessRequestService::new(
      db_service.clone(),
      auth_service.clone(),
      secret_service,
      time_service.clone(),
      "http://localhost:1135".to_string(),
    ));

  let app_service = builder
    .auth_service(auth_service)
    .with_tool_service(tool_service)
    .access_request_service(access_request_service)
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  Ok(TestHarness { state, db_service })
}

// ============================================================================
// Helper: seed a draft access request + enabled toolset instance in DB
// ============================================================================

async fn seed_draft_request(
  db_service: &dyn DbService,
  request_id: &str,
  flow_type: &str,
  redirect_uri: Option<&str>,
) -> anyhow::Result<AppAccessRequestRow> {
  let now = chrono::Utc::now().timestamp();
  let row = AppAccessRequestRow {
    id: request_id.to_string(),
    app_client_id: "test-app-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: flow_type.to_string(),
    redirect_uri: redirect_uri.map(|u| u.to_string()),
    status: "draft".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: now + 600,
    created_at: now,
    updated_at: now,
  };
  Ok(db_service.create(&row).await?)
}

async fn seed_toolset_instance(
  db_service: &dyn DbService,
  instance_id: &str,
  user_id: &str,
  enabled: bool,
  has_api_key: bool,
) -> anyhow::Result<()> {
  let now = chrono::Utc::now().timestamp();
  let row = ToolsetRow {
    id: instance_id.to_string(),
    user_id: user_id.to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    slug: "my-exa-instance".to_string(),
    description: None,
    enabled,
    encrypted_api_key: if has_api_key {
      Some("encrypted_key".to_string())
    } else {
      None
    },
    salt: if has_api_key {
      Some("salt".to_string())
    } else {
      None
    },
    nonce: if has_api_key {
      Some("nonce".to_string())
    } else {
      None
    },
    created_at: now,
    updated_at: now,
  };
  db_service.create_toolset(&row).await?;
  Ok(())
}

// ============================================================================
// approve_access_request_handler - success
// ============================================================================

#[rstest]
#[case::popup_flow("popup", None, false)]
#[case::redirect_flow("redirect", Some("https://app.com/cb"), true)]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_success(
  #[case] flow_type: &str,
  #[case] redirect_url: Option<&str>,
  #[case] expect_redirect: bool,
) -> anyhow::Result<()> {
  let request_id = "ar-approve-ok";
  let user_id = "test-user-1";
  let instance_id = "toolset-instance-1";

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_register_access_request_consent()
    .times(1)
    .returning(move |_, _, _, _| {
      Ok(RegisterAccessRequestConsentResponse {
        scope: "scope_resource:ar-approve-ok".to_string(),
        access_request_id: "ar-approve-ok".to_string(),
        access_request_scope: "scope_access_request:ar-approve-ok".to_string(),
      })
    });

  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    flow_type,
    redirect_url,
  )
  .await?;
  seed_toolset_instance(
    harness.db_service.as_ref(),
    instance_id,
    user_id,
    true,
    true,
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved": {
      "toolsets": [{
        "toolset_type": "builtin-exa-search",
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
  assert_eq!("approved", result.status);
  assert_eq!(flow_type, result.flow_type);
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
// approve_access_request_handler - error: instance not owned
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_instance_not_owned() -> anyhow::Result<()> {
  let request_id = "ar-not-owned";
  let user_id = "test-user-2";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "popup", None).await?;
  // No toolset instance seeded for this user -> "not owned"

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved": {
      "toolsets": [{
        "toolset_type": "builtin-exa-search",
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
    "app_access_request_error-tool_instance_not_owned",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// approve_access_request_handler - error: instance not enabled
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_instance_not_enabled() -> anyhow::Result<()> {
  let request_id = "ar-not-enabled";
  let user_id = "test-user-3";
  let instance_id = "disabled-instance";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "popup", None).await?;
  seed_toolset_instance(
    harness.db_service.as_ref(),
    instance_id,
    user_id,
    false, // not enabled
    true,
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved": {
      "toolsets": [{
        "toolset_type": "builtin-exa-search",
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
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "app_access_request_error-tool_instance_not_configured",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// approve_access_request_handler - error: instance no API key
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_access_request_instance_no_api_key() -> anyhow::Result<()> {
  let request_id = "ar-no-key";
  let user_id = "test-user-4";
  let instance_id = "no-key-instance";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(harness.db_service.as_ref(), request_id, "popup", None).await?;
  seed_toolset_instance(
    harness.db_service.as_ref(),
    instance_id,
    user_id,
    true,
    false, // no API key
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved": {
      "toolsets": [{
        "toolset_type": "builtin-exa-search",
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
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "app_access_request_error-tool_instance_not_configured",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// deny_access_request_handler - success
// ============================================================================

#[rstest]
#[case::popup_flow("popup", None, false)]
#[case::redirect_flow("redirect", Some("https://app.com/cb"), true)]
#[tokio::test]
#[anyhow_trace]
async fn test_deny_access_request_success(
  #[case] flow_type: &str,
  #[case] redirect_url: Option<&str>,
  #[case] expect_redirect: bool,
) -> anyhow::Result<()> {
  let request_id = &format!("ar-deny-{}", flow_type);
  let user_id = "test-user-5";

  let mock_auth = MockAuthService::default();
  let harness = build_test_harness(mock_auth).await?;
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    flow_type,
    redirect_url,
  )
  .await?;

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_DENY,
      post(deny_access_request_handler),
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
  assert_eq!("denied", result.status);
  assert_eq!(flow_type, result.flow_type);
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
// Auth tier tests (inline with this module's handler tests)
// ============================================================================

#[anyhow_trace]
#[rstest]
#[case::app_review("GET", "/bodhi/v1/access-requests/test-id/review")]
#[case::app_approve("PUT", "/bodhi/v1/access-requests/test-id/approve")]
#[case::app_deny("POST", "/bodhi/v1/access-requests/test-id/deny")]
#[tokio::test]
async fn test_app_access_request_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}
