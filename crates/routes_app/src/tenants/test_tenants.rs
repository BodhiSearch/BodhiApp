use crate::test_utils::RequestAuthContextExt;
use crate::{tenants_activate, tenants_create, tenants_index, ENDPOINT_TENANTS};
use anyhow_trace::anyhow_trace;
use axum::body::{to_bytes, Body};
use axum::{
  http::{status::StatusCode, Request},
  routing::{get, post},
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
  AppService, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_DEPLOYMENT, BODHI_HOST, BODHI_PORT,
  BODHI_SCHEME,
};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

async fn build_tenants_router(app_service: Arc<dyn AppService>) -> Router {
  let state = app_service.clone();
  Router::new()
    .route(ENDPOINT_TENANTS, get(tenants_index))
    .route(ENDPOINT_TENANTS, post(tenants_create))
    .route(
      &format!("{ENDPOINT_TENANTS}/{{client_id}}/activate"),
      post(tenants_activate),
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_index_returns_error_when_not_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_standalone_app_service(&temp_bodhi_home).await?;
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::get(ENDPOINT_TENANTS)
        .body(Body::empty())?
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
async fn test_tenants_create_returns_error_when_not_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_standalone_app_service(&temp_bodhi_home).await?;
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_TENANTS)
        .json(json! {{"name": "Test Tenant", "description": "A test tenant"}})?
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
async fn test_tenants_activate_returns_error_when_not_multi_tenant(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_service = build_standalone_app_service(&temp_bodhi_home).await?;
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(&format!("{ENDPOINT_TENANTS}/some-client-id/activate"))
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
