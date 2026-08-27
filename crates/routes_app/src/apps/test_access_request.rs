use crate::test_utils::RequestAuthContextExt;
use crate::{
  apps::{AppAccessSummary, ListAppAccessResponse},
  apps_get_consent_context, apps_list_user_access, apps_revoke_access_request, apps_submit_consent,
  ResourceAccess, ENDPOINT_ACCESS_REQUESTS_APPS, ENDPOINT_ACCESS_REQUESTS_REVOKE,
  ENDPOINT_APPS_ACCESS_REQUESTS, ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::StatusCode,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use services::{
  test_utils::{
    approved_request, make_request, AppServiceStubBuilder, FrozenTimeService, TEST_TENANT_ID,
  },
  AppAccessRequest, AppAccessRequestStatus, AppClientInfo, AuthContext, AuthServiceError,
  DbService, DefaultAccessRequestService, MockAuthService, ResourceRole,
  SCOPE_ACCESS_REQUEST_PREFIX,
};
use std::sync::Arc;
use tower::ServiceExt;

const APP_CLIENT_ID: &str = "app-acme";
const REDIRECT_URI: &str = "https://acme.dev/cb";
/// The session resource client id baked into `AuthContext::test_session*` factories.
const RESOURCE_CLIENT_ID: &str = "test-client-id";
const AUTHORIZE_ENDPOINT: &str = "https://kc.example.com/realms/bodhi/protocol/openid-connect/auth";

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
  let access_request_service: Arc<dyn services::AccessRequestService> = Arc::new(
    DefaultAccessRequestService::new(db_service.clone(), auth_service.clone(), time_service),
  );

  let app_service = builder
    .auth_service(auth_service)
    .access_request_service(access_request_service)
    .build()
    .await?;

  Ok(TestHarness {
    state: Arc::new(app_service),
    db_service,
  })
}

/// MockAuthService resolving `app-acme` with `https://acme.dev/cb` registered.
fn consent_mock_auth() -> MockAuthService {
  let mut mock = MockAuthService::default();
  mock.expect_get_app_client_info().returning(|_, _| {
    Ok(AppClientInfo {
      name: "Acme App".to_string(),
      description: "Acme test app".to_string(),
      redirect_uris: Some(vec![REDIRECT_URI.to_string()]),
    })
  });
  mock
}

fn consent_mock_auth_with_authorize() -> MockAuthService {
  let mut mock = consent_mock_auth();
  mock
    .expect_authorize_url()
    .return_const(AUTHORIZE_ENDPOINT.to_string());
  mock
}

fn consent_router(state: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(
      ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT,
      get(apps_get_consent_context),
    )
    .route(ENDPOINT_APPS_ACCESS_REQUESTS, post(apps_submit_consent))
    .with_state(state)
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

fn base_pairs() -> Vec<(&'static str, String)> {
  vec![
    ("client_id", APP_CLIENT_ID.to_string()),
    ("redirect_uri", REDIRECT_URI.to_string()),
    ("response_type", "code".to_string()),
    ("state", "st-123".to_string()),
    ("code_challenge", "ch-456".to_string()),
    ("code_challenge_method", "S256".to_string()),
  ]
}

fn encode_query(pairs: &[(&str, String)]) -> String {
  let mut serializer = url::form_urlencoded::Serializer::new(String::new());
  for (key, value) in pairs {
    serializer.append_pair(key, value);
  }
  serializer.finish()
}

fn consent_query() -> String {
  encode_query(&base_pairs())
}

fn consent_query_with(extra: &[(&str, &str)]) -> String {
  let mut pairs = base_pairs();
  for (key, value) in extra {
    pairs.push((key, value.to_string()));
  }
  encode_query(&pairs)
}

fn get_consent_request(query: &str, ctx: AuthContext) -> axum::http::Request<Body> {
  axum::http::Request::builder()
    .method("GET")
    .uri(format!("{ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT}?{query}"))
    .body(Body::empty())
    .unwrap()
    .with_auth_context(ctx)
}

fn post_consent_request(
  body: &Value,
  ctx: AuthContext,
) -> anyhow::Result<axum::http::Request<Body>> {
  Ok(
    axum::http::Request::builder()
      .method("POST")
      .uri(ENDPOINT_APPS_ACCESS_REQUESTS)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(body)?))?
      .with_auth_context(ctx),
  )
}

fn user_session(user_id: &str) -> AuthContext {
  AuthContext::test_session_with_token(user_id, "user@test.com", ResourceRole::User, "dummy-token")
}

fn session_with_role(user_id: &str, role: ResourceRole) -> AuthContext {
  AuthContext::test_session_with_token(user_id, "user@test.com", role, "dummy-token")
}

async fn seed_approved(
  db_service: &dyn DbService,
  id: &str,
  user_id: &str,
  app_client_id: &str,
  approved_json: &str,
  created_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<AppAccessRequest> {
  let row = AppAccessRequest {
    app_client_id: app_client_id.to_string(),
    approved: Some(approved_json.to_string()),
    access_request_scope: Some(format!(
      "{SCOPE_ACCESS_REQUEST_PREFIX}{RESOURCE_CLIENT_ID}.{id}"
    )),
    ..approved_request(id, TEST_TENANT_ID, user_id, created_at)
  };
  Ok(db_service.create(&row).await?)
}

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

// ---------------------------------------------------------------------------
// Consent context (GET)
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_happy_path_defaults() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let response = router
    .oneshot(get_consent_request(
      // Empty scope param → role User, both sections requested.
      &consent_query_with(&[("scope", "")]),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("ok", body["result"].as_str().unwrap());
  assert_eq!(APP_CLIENT_ID, body["app"]["client_id"].as_str().unwrap());
  assert_eq!("Acme App", body["app"]["name"].as_str().unwrap());
  assert_eq!(
    "Acme test app",
    body["app"]["description"].as_str().unwrap()
  );
  assert_eq!(REDIRECT_URI, body["app"]["redirect_uri"].as_str().unwrap());
  assert_eq!("scope_user_user", body["scope"]["role"].as_str().unwrap());
  assert_eq!(true, body["scope"]["llms"].as_bool().unwrap());
  assert_eq!(true, body["scope"]["mcps"].as_bool().unwrap());
  assert_eq!(0, body["scope"]["passthrough"].as_array().unwrap().len());
  assert_eq!(true, body["can_approve"].as_bool().unwrap());
  assert!(body.get("prior_grant").is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_guest_cannot_approve() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let response = router
    .oneshot(get_consent_request(
      &consent_query(),
      session_with_role("guest-1", ResourceRole::Guest),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("ok", body["result"].as_str().unwrap());
  assert_eq!(false, body["can_approve"].as_bool().unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_prior_grant_explicit() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  seed_approved(
    harness.db_service.as_ref(),
    "ar-prior",
    "owner-1",
    APP_CLIENT_ID,
    r#"{"version":"1","models_list":true,"models_access":{"type":"specific","ids":["alias-x"]}}"#,
    chrono::Utc::now(),
  )
  .await?;

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &consent_query_with(&[("source_access_request_id", "ar-prior")]),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("ok", body["result"].as_str().unwrap());
  assert_eq!("ar-prior", body["prior_grant"]["id"].as_str().unwrap());
  assert_eq!("explicit", body["prior_grant"]["source"].as_str().unwrap());
  assert_eq!(
    "scope_user_user",
    body["prior_grant"]["approved_role"].as_str().unwrap()
  );
  assert_eq!(
    "alias-x",
    body["prior_grant"]["approved"]["models_access"]["ids"][0]
      .as_str()
      .unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_prior_grant_latest() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let now = chrono::Utc::now();
  seed_approved(
    harness.db_service.as_ref(),
    "ar-older",
    "owner-1",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    now - chrono::Duration::hours(2),
  )
  .await?;
  seed_approved(
    harness.db_service.as_ref(),
    "ar-newer",
    "owner-1",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    now,
  )
  .await?;

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &consent_query(),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("ar-newer", body["prior_grant"]["id"].as_str().unwrap());
  assert_eq!("latest", body["prior_grant"]["source"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_non_matching_source_ignored_no_latest_fallback(
) -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let now = chrono::Utc::now();
  // A grant belonging to another user, named explicitly as the source.
  seed_approved(
    harness.db_service.as_ref(),
    "ar-foreign",
    "someone-else",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    now,
  )
  .await?;
  // The caller has a latest grant, but an explicit non-matching source must NOT fall back.
  seed_approved(
    harness.db_service.as_ref(),
    "ar-mine",
    "owner-1",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    now,
  )
  .await?;

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &consent_query_with(&[("source_access_request_id", "ar-foreign")]),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("ok", body["result"].as_str().unwrap());
  assert!(body.get("prior_grant").is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_unknown_client_error_union_no_redirect() -> anyhow::Result<()> {
  let mut mock_auth = MockAuthService::default();
  mock_auth.expect_get_app_client_info().returning(|_, _| {
    Err(AuthServiceError::AuthServiceApiError {
      status: 404,
      body: "client not found".to_string(),
    })
  });
  let harness = build_test_harness(mock_auth).await?;

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &consent_query(),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("error", body["result"].as_str().unwrap());
  assert_eq!("unauthorized_client", body["error"].as_str().unwrap());
  assert!(body.get("redirect_url").is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_redirect_uri_mismatch_error_union_no_redirect(
) -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let mut pairs = base_pairs();
  pairs[1] = ("redirect_uri", "https://evil.dev/cb".to_string());

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &encode_query(&pairs),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("error", body["result"].as_str().unwrap());
  assert_eq!("invalid_request", body["error"].as_str().unwrap());
  assert!(body.get("redirect_url").is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_consent_context_invalid_scope_error_union_with_redirect() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;

  let router = consent_router(harness.state);
  let response = router
    .oneshot(get_consent_request(
      &consent_query_with(&[("scope", "scope_apps:garbage")]),
      user_session("owner-1"),
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("error", body["result"].as_str().unwrap());
  assert_eq!("invalid_scope", body["error"].as_str().unwrap());
  let redirect_url = body["redirect_url"].as_str().unwrap();
  assert!(redirect_url.starts_with(REDIRECT_URI));
  assert!(redirect_url.contains("error_source=bodhi"));
  assert!(redirect_url.contains("state=st-123"));
  Ok(())
}

// ---------------------------------------------------------------------------
// Submit consent (POST) — approve
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_happy_path() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth_with_authorize()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query(),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<Value>().await?;
  let id = body["id"].as_str().expect("approve returns the row id");
  let expected_scope =
    format!("openid profile email roles {SCOPE_ACCESS_REQUEST_PREFIX}{RESOURCE_CLIENT_ID}.{id}");
  let expected_url = format!(
    "{AUTHORIZE_ENDPOINT}?response_type=code&client_id=app-acme\
     &redirect_uri=https%3A%2F%2Facme.dev%2Fcb&state=st-123&code_challenge=ch-456\
     &code_challenge_method=S256&scope=openid+profile+email+roles+scope_access_request%3A{RESOURCE_CLIENT_ID}.{id}"
  );
  assert_eq!(expected_url, body["redirect_url"].as_str().unwrap());

  let url = url::Url::parse(body["redirect_url"].as_str().unwrap())?;
  let pairs: Vec<(String, String)> = url
    .query_pairs()
    .map(|(k, v)| (k.into_owned(), v.into_owned()))
    .collect();
  assert!(pairs.contains(&("response_type".to_string(), "code".to_string())));
  assert!(pairs.contains(&("client_id".to_string(), APP_CLIENT_ID.to_string())));
  assert!(pairs.contains(&("redirect_uri".to_string(), REDIRECT_URI.to_string())));
  assert!(pairs.contains(&("state".to_string(), "st-123".to_string())));
  assert!(pairs.contains(&("code_challenge".to_string(), "ch-456".to_string())));
  assert!(pairs.contains(&("code_challenge_method".to_string(), "S256".to_string())));
  assert!(pairs.contains(&("scope".to_string(), expected_scope.clone())));

  let row = harness
    .db_service
    .get(TEST_TENANT_ID, id)
    .await?
    .expect("approved row persisted");
  assert_eq!(AppAccessRequestStatus::Approved, row.status);
  assert_eq!(Some("approver-1".to_string()), row.user_id);
  assert_eq!(
    Some(format!(
      "{SCOPE_ACCESS_REQUEST_PREFIX}{RESOURCE_CLIENT_ID}.{id}"
    )),
    row.access_request_scope
  );
  assert_eq!(None, row.source_access_request_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_passthrough_scope_survives() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth_with_authorize()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("scope", "scope_user_user offline_access")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<Value>().await?;
  let id = body["id"].as_str().unwrap();
  let url = url::Url::parse(body["redirect_url"].as_str().unwrap())?;
  let scope = url
    .query_pairs()
    .find(|(k, _)| k == "scope")
    .map(|(_, v)| v.into_owned())
    .expect("scope param present");
  assert_eq!(
    format!("openid profile email roles offline_access {SCOPE_ACCESS_REQUEST_PREFIX}{RESOURCE_CLIENT_ID}.{id}"),
    scope
  );
  Ok(())
}

#[rstest]
#[case::approved_exceeds_requested("", ResourceRole::PowerUser, "scope_user_power_user")]
#[case::approved_exceeds_approver_ceiling(
  "scope_user_power_user",
  ResourceRole::User,
  "scope_user_power_user"
)]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_privilege_escalation_forbidden(
  #[case] requested_scope: &str,
  #[case] approver_role: ResourceRole,
  #[case] approved_role: &str,
) -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("scope", requested_scope)]),
    "decision": "approve",
    "approved_role": approved_role,
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(
      &body,
      session_with_role("approver-1", approver_role),
    )?)
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-privilege_escalation",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_valid_downgrade() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth_with_authorize()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("scope", "scope_user_power_user")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(
      &body,
      session_with_role("approver-1", ResourceRole::PowerUser),
    )?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<Value>().await?;
  assert!(body["id"].as_str().is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_guest_forbidden() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query(),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(
      &body,
      session_with_role("guest-1", ResourceRole::Guest),
    )?)
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-insufficient_privileges",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[case::missing_approved_role(json!({
  "query": "", "decision": "approve", "approved": {"version": "1"}
}))]
#[case::missing_approved(json!({
  "query": "", "decision": "approve", "approved_role": "scope_user_user"
}))]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_missing_fields_bad_request(
  #[case] mut body: Value,
) -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);
  body["query"] = Value::String(consent_query());

  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-consent_field_missing",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_grant_exceeds_scope_rejected() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  // Scope suppressed MCPs, yet the grant carries an MCP approval.
  let body = json!({
    "query": consent_query_with(&[("scope", "scope_apps:mcps:false")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {
      "version": "1",
      "mcps": [{
        "url": "https://mcp.example.com/mcp",
        "status": "approved",
        "instance": {"id": "any-instance"}
      }]
    }
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "app_scope_error-grant_exceeds_scope",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_mcp_instance_not_owned() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query(),
    "decision": "approve",
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
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-mcp_instance_not_owned",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_mcp_instance_disabled() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let instance_id = seed_mcp_instance(
    harness.state.as_ref(),
    "approver-1",
    "https://mcp.example.com/mcp",
    "disabled-tool",
    false,
  )
  .await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query(),
    "decision": "approve",
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
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-mcp_instance_not_configured",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_stores_resolving_source_id() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth_with_authorize()).await?;
  seed_approved(
    harness.db_service.as_ref(),
    "ar-prior",
    "approver-1",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    chrono::Utc::now(),
  )
  .await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("source_access_request_id", "ar-prior")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<Value>().await?;
  let id = body["id"].as_str().unwrap();
  let row = harness
    .db_service
    .get(TEST_TENANT_ID, id)
    .await?
    .expect("approved row persisted");
  assert_eq!(Some("ar-prior".to_string()), row.source_access_request_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_approve_drops_non_resolving_source_id() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth_with_authorize()).await?;
  // The named source belongs to a different user — must not be stored.
  seed_approved(
    harness.db_service.as_ref(),
    "ar-foreign",
    "someone-else",
    APP_CLIENT_ID,
    r#"{"version":"1"}"#,
    chrono::Utc::now(),
  )
  .await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("source_access_request_id", "ar-foreign")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<Value>().await?;
  let id = body["id"].as_str().unwrap();
  let row = harness
    .db_service
    .get(TEST_TENANT_ID, id)
    .await?
    .expect("approved row persisted");
  assert_eq!(None, row.source_access_request_id);
  Ok(())
}

// ---------------------------------------------------------------------------
// Submit consent (POST) — deny + query failures
// ---------------------------------------------------------------------------

#[rstest]
#[case::user(ResourceRole::User)]
#[case::guest(ResourceRole::Guest)]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_deny_redirects_with_access_denied(
  #[case] role: ResourceRole,
) -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query(),
    "decision": "deny"
  });
  let response = router
    .oneshot(post_consent_request(
      &body,
      session_with_role("denier-1", role),
    )?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert!(body.get("id").is_none(), "deny creates no row");
  let redirect_url = body["redirect_url"].as_str().unwrap();
  assert!(redirect_url.starts_with(REDIRECT_URI));
  let url = url::Url::parse(redirect_url)?;
  let pairs: Vec<(String, String)> = url
    .query_pairs()
    .map(|(k, v)| (k.into_owned(), v.into_owned()))
    .collect();
  assert!(pairs.contains(&("error".to_string(), "access_denied".to_string())));
  assert!(pairs.contains(&("error_source".to_string(), "bodhi".to_string())));
  assert!(pairs.contains(&("state".to_string(), "st-123".to_string())));
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_redirect_mismatch_rejected_in_app() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let mut pairs = base_pairs();
  pairs[1] = ("redirect_uri", "https://evil.dev/cb".to_string());
  let body = json!({
    "query": encode_query(&pairs),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "apps_route_error-consent_rejected",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_submit_consent_redirectable_bad_query_returns_error_redirect() -> anyhow::Result<()> {
  let harness = build_test_harness(consent_mock_auth()).await?;
  let router = consent_router(harness.state);

  let body = json!({
    "query": consent_query_with(&[("scope", "scope_apps:garbage")]),
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1"}
  });
  let response = router
    .oneshot(post_consent_request(&body, user_session("approver-1"))?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert!(body.get("id").is_none(), "no row on a redirected error");
  let redirect_url = body["redirect_url"].as_str().unwrap();
  assert!(redirect_url.starts_with(REDIRECT_URI));
  assert!(redirect_url.contains("error=invalid_scope"));
  assert!(redirect_url.contains("error_source=bodhi"));
  Ok(())
}

// ---------------------------------------------------------------------------
// List + revoke (unchanged endpoints)
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_user_access_returns_only_callers_approved_with_summary() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  seed_approved(
    harness.db_service.as_ref(),
    "ar-mine",
    "owner-1",
    "app-mine",
    r#"{"version":"1","models_list":true,"models_access":{"type":"specific","ids":["alias-x"]},"mcps_list":false,"mcps_access":{"type":"specific","ids":["mcp-1"]}}"#,
    chrono::Utc::now(),
  )
  .await?;
  // Another user's grant must NOT appear.
  seed_approved(
    harness.db_service.as_ref(),
    "ar-other",
    "owner-2",
    "app-other",
    r#"{"version":"1"}"#,
    chrono::Utc::now(),
  )
  .await?;

  let router = management_router(harness.state);
  let request = axum::http::Request::builder()
    .method("GET")
    .uri(ENDPOINT_ACCESS_REQUESTS_APPS)
    .body(Body::empty())?
    .with_auth_context(user_session("owner-1"));
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_revoke_access_request_transitions_to_revoked() -> anyhow::Result<()> {
  let harness = build_test_harness(MockAuthService::default()).await?;
  seed_approved(
    harness.db_service.as_ref(),
    "ar-revoke",
    "owner-1",
    "app-1",
    r#"{"version":"1"}"#,
    chrono::Utc::now(),
  )
  .await?;

  let router = management_router(harness.state.clone());
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-revoke/revoke")
    .body(Body::empty())?
    .with_auth_context(user_session("owner-1"));
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
    .with_auth_context(user_session("owner-1"));
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
  seed_approved(
    harness.db_service.as_ref(),
    "ar-revoke-2",
    "owner-1",
    "app-1",
    r#"{"version":"1"}"#,
    chrono::Utc::now(),
  )
  .await?;

  let router = management_router(harness.state);
  // A different user attempts the revoke.
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-revoke-2/revoke")
    .body(Body::empty())?
    .with_auth_context(user_session("attacker"));
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
  let row = make_request("ar-draft-revoke", TEST_TENANT_ID, chrono::Utc::now());
  harness.db_service.create(&row).await?;

  let router = management_router(harness.state);
  let request = axum::http::Request::builder()
    .method("POST")
    .uri("/bodhi/v1/access-requests/ar-draft-revoke/revoke")
    .body(Body::empty())?
    .with_auth_context(user_session("owner-1"));
  let response = router.oneshot(request).await?;
  assert_ne!(StatusCode::OK, response.status());
  Ok(())
}

#[test]
fn app_access_summary_clamps_tampered_approved_role() {
  use services::UserScope;
  let ts: chrono::DateTime<chrono::Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
  let row = |approved_role: Option<&str>| AppAccessRequest {
    requested_role: "scope_user_power_user".to_string(),
    approved_role: approved_role.map(|s| s.to_string()),
    approved: None,
    ..approved_request("ar-1", TEST_TENANT_ID, "u", ts)
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
