use crate::test_utils::RequestAuthContextExt;
use crate::{
  dashboard_auth_callback, dashboard_auth_initiate, RedirectResponse,
  ENDPOINT_DASHBOARD_AUTH_CALLBACK, ENDPOINT_DASHBOARD_AUTH_INITIATE,
};
use anyhow_trace::anyhow_trace;
use axum::body::to_bytes;
use axum::{
  http::{status::StatusCode, Request},
  routing::post,
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::RequestTestExt;
use services::test_utils::temp_bodhi_home;
use services::AuthContext;
use services::{
  test_utils::{AppServiceStubBuilder, SettingServiceStub},
  AppService, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_DEPLOYMENT, BODHI_MULTITENANT_CLIENT_ID,
};
use services::{BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;
use tower_sessions::SessionStore;

async fn build_dashboard_router(app_service: Arc<dyn AppService>) -> Router {
  let state = app_service.clone();
  Router::new()
    .route(
      ENDPOINT_DASHBOARD_AUTH_INITIATE,
      post(dashboard_auth_initiate),
    )
    .route(
      ENDPOINT_DASHBOARD_AUTH_CALLBACK,
      post(dashboard_auth_callback),
    )
    .layer(app_service.session_service().session_layer())
    .with_state(state)
}

async fn build_standalone_app_service(
  temp_bodhi_home: &TempDir,
) -> anyhow::Result<Arc<dyn AppService>> {
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "localhost".to_string()),
    (BODHI_PORT.to_string(), "3000".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
    (BODHI_DEPLOYMENT.to_string(), "standalone".to_string()),
  ]));
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  let app_service = builder.build().await?;
  Ok(Arc::new(app_service))
}

async fn build_multitenant_app_service(
  temp_bodhi_home: &TempDir,
) -> anyhow::Result<Arc<dyn AppService>> {
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
      (
        BODHI_AUTH_URL.to_string(),
        "http://test-id.getbodhi.app".to_string(),
      ),
      (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
      (BODHI_DEPLOYMENT.to_string(), "multi_tenant".to_string()),
      (
        BODHI_MULTITENANT_CLIENT_ID.to_string(),
        "dashboard-client-id".to_string(),
      ),
    ]))
    .await;
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  let app_service = builder.build().await?;
  Ok(Arc::new(app_service))
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_initiate_returns_error_when_not_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_standalone_app_service(&temp_bodhi_home).await?;
  let router = build_dashboard_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_DASHBOARD_AUTH_INITIATE)
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
        }),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: Value = serde_json::from_slice(&body_bytes)?;
  assert_eq!(
    "dashboard_auth_route_error-not_multi_tenant",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_callback_returns_error_when_not_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_standalone_app_service(&temp_bodhi_home).await?;
  let router = build_dashboard_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_DASHBOARD_AUTH_CALLBACK)
        .json(json! {{"code": "test_code", "state": "test_state"}})?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
        }),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: Value = serde_json::from_slice(&body_bytes)?;
  assert_eq!(
    "dashboard_auth_route_error-not_multi_tenant",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_initiate_returns_error_when_client_config_missing(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // Multi-tenant mode but no client ID/secret configured
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "localhost".to_string()),
    (BODHI_PORT.to_string(), "3000".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
    (BODHI_DEPLOYMENT.to_string(), "multi_tenant".to_string()),
    // No BODHI_MULTITENANT_CLIENT_ID or SECRET
  ]));
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  let app_service = Arc::new(builder.build().await?);
  let router = build_dashboard_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_DASHBOARD_AUTH_INITIATE)
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
        }),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: Value = serde_json::from_slice(&body_bytes)?;
  assert_eq!(
    "dashboard_auth_route_error-missing_client_config",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_initiate_returns_redirect_url_in_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_multitenant_app_service(&temp_bodhi_home).await?;
  let router = build_dashboard_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_DASHBOARD_AUTH_INITIATE)
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
        }),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::CREATED, status);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

  // Should start with the auth URL
  let expected_prefix =
    "http://test-id.getbodhi.app/realms/test-realm/protocol/openid-connect/auth";
  assert!(
    body.location.starts_with(expected_prefix),
    "Expected location to start with '{}', got '{}'",
    expected_prefix,
    body.location
  );

  // Parse and verify query params
  let url = url::Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  assert_eq!(
    Some("code"),
    query_params.get("response_type").map(|s| s.as_str())
  );
  assert_eq!(
    Some("dashboard-client-id"),
    query_params.get("client_id").map(|s| s.as_str())
  );
  assert!(query_params.contains_key("state"));
  assert!(query_params.contains_key("code_challenge"));
  assert_eq!(
    Some("S256"),
    query_params
      .get("code_challenge_method")
      .map(|s| s.as_str())
  );
  assert_eq!(
    Some("openid email profile"),
    query_params.get("scope").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_callback_validates_state_mismatch(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_multitenant_app_service(&temp_bodhi_home).await?;
  let session_service = app_service.session_service();

  // First, set up session with a stored state
  let id = tower_sessions::session::Id::default();
  let mut record = tower_sessions::session::Record {
    id,
    data: maplit::hashmap! {
      "dashboard_oauth_state".to_string() => Value::String("correct_state".to_string()),
      "dashboard_pkce_verifier".to_string() => Value::String("test_verifier".to_string()),
      "dashboard_callback_url".to_string() => Value::String("http://localhost:3000/ui/auth/callback".to_string()),
    },
    expiry_date: time::OffsetDateTime::now_utc() + time::Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let router = build_dashboard_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_DASHBOARD_AUTH_CALLBACK)
        .header("Cookie", format!("bodhiapp_session_id={}", record.id))
        .header("Sec-Fetch-Site", "same-origin")
        .json(json! {{"code": "test_code", "state": "wrong_state"}})?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
        }),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::BAD_REQUEST, status);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: Value = serde_json::from_slice(&body_bytes)?;
  assert_eq!(
    "dashboard_auth_route_error-state_digest_mismatch",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}
