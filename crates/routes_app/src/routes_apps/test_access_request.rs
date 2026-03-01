use crate::{
  approve_access_request_handler, deny_access_request_handler,
  routes_apps::AccessRequestActionResponse, ENDPOINT_ACCESS_REQUESTS_APPROVE,
  ENDPOINT_ACCESS_REQUESTS_DENY,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{body::Body, http::StatusCode, routing::put};
use axum::{routing::post, Router};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::AppAccessRequestStatus;
use services::ResourceRole;
use services::{
  db::DbService,
  test_utils::{AppServiceStubBuilder, FrozenTimeService},
  DefaultAccessRequestService, DefaultToolService, MockAuthService, MockExaService,
  RegisterAccessRequestConsentResponse, {AppAccessRequestRow, FlowType, ToolsetRow},
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
  builder
    .with_app_instance(services::AppInstance::test_default())
    .await;
  let access_request_service: Arc<dyn services::AccessRequestService> =
    Arc::new(DefaultAccessRequestService::new(
      db_service.clone(),
      auth_service.clone(),
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
  flow_type: FlowType,
  redirect_uri: Option<&str>,
  requested_role: &str,
) -> anyhow::Result<AppAccessRequestRow> {
  let now = chrono::Utc::now();
  let row = AppAccessRequestRow {
    id: request_id.to_string(),
    app_client_id: "test-app-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type,
    redirect_uri: redirect_uri.map(|u| u.to_string()),
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
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
  Ok(db_service.create(&row).await?)
}

async fn seed_toolset_instance(
  db_service: &dyn DbService,
  instance_id: &str,
  user_id: &str,
  enabled: bool,
  has_api_key: bool,
) -> anyhow::Result<()> {
  let now = chrono::Utc::now();
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
  let instance_id = "toolset-instance-1";

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
    "approved_role": "scope_user_user",
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
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;
  // No toolset instance seeded for this user -> "not owned"

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_user",
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
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;
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
    "approved_role": "scope_user_user",
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
  seed_draft_request(
    harness.db_service.as_ref(),
    request_id,
    FlowType::Popup,
    None,
    "scope_user_user",
  )
  .await?;
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
    "approved_role": "scope_user_user",
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
// approve_access_request_handler - error: privilege escalation (user grants power_user)
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
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_power_user",
    "approved": {
      "toolsets": [],
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
    "app_access_request_error-privilege_escalation",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// approve_access_request_handler - success: power_user downgrades scope_user_power_user to scope_user_user
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_valid_downgrade_power_user_grants_user() -> anyhow::Result<()> {
  let request_id = "ar-downgrade";
  let user_id = "test-user-7";
  let instance_id = "downgrade-instance";

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
    "approved_role": "scope_user_user",
    "approved": {
      "toolsets": [{
        "toolset_type": "builtin-exa-search",
        "status": "approved",
        "instance": {"id": instance_id}
      }]
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
// approve_access_request_handler - error: approved_role exceeds requested_role
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
      put(approve_access_request_handler),
    )
    .with_state(harness.state);

  let body = json!({
    "approved_role": "scope_user_power_user",
    "approved": {
      "toolsets": [],
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
    "app_access_request_error-privilege_escalation",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}
