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
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::AuthContext;
use services::{
  test_utils::AppServiceStubBuilder, AppService, AppStatus, KcCreateTenantResponse, MockAuthService,
};
use std::sync::Arc;
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

// --- Standalone error tests ---

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_index_returns_error_when_not_multi_tenant() -> anyhow::Result<()> {
  let app_service = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::get(ENDPOINT_TENANTS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
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
async fn test_tenants_create_returns_error_when_not_multi_tenant() -> anyhow::Result<()> {
  let app_service = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_TENANTS)
        .json(json! {{"name": "Test Tenant", "description": "A test tenant"}})?
        .with_auth_context(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
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
async fn test_tenants_activate_returns_error_when_not_multi_tenant() -> anyhow::Result<()> {
  let app_service = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = build_tenants_router(app_service).await;

  let resp = router
    .oneshot(
      Request::post(&format!("{ENDPOINT_TENANTS}/some-client-id/activate"))
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
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

// --- Multi-tenant happy-path tests ---

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_index_returns_user_tenants_for_multi_tenant_session() -> anyhow::Result<()> {
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_multitenant_settings()
      .await
      .build()
      .await?,
  );

  // Create a tenant with status Ready
  let tenant = app_service
    .tenant_service()
    .create_tenant(
      "test-client-id",
      "test-client-secret",
      "Test Tenant",
      Some("A test tenant".to_string()),
      AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;

  // Create tenant-user membership
  app_service
    .tenant_service()
    .upsert_tenant_user(&tenant.id, "test-user")
    .await?;

  let router = build_tenants_router(app_service).await;

  let auth_context = AuthContext::test_multi_tenant_session("test-user", "test@example.com");

  let resp = router
    .oneshot(
      Request::get(ENDPOINT_TENANTS)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: Value = resp.json().await?;
  let tenants = body["tenants"].as_array().unwrap();
  assert_eq!(1, tenants.len());
  assert_eq!("test-client-id", tenants[0]["client_id"].as_str().unwrap());
  assert_eq!("Test Tenant", tenants[0]["name"].as_str().unwrap());
  assert_eq!("ready", tenants[0]["status"].as_str().unwrap());
  assert_eq!(false, tenants[0]["is_active"].as_bool().unwrap());
  assert_eq!(false, tenants[0]["logged_in"].as_bool().unwrap());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_index_with_client_id_some_returns_tenants() -> anyhow::Result<()> {
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_multitenant_settings()
      .await
      .build()
      .await?,
  );

  // Create a tenant with status Ready
  let tenant = app_service
    .tenant_service()
    .create_tenant(
      "test-client-id",
      "test-client-secret",
      "Test Tenant",
      Some("A test tenant".to_string()),
      AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;

  // Create tenant-user membership
  app_service
    .tenant_service()
    .upsert_tenant_user(&tenant.id, "test-user")
    .await?;

  let router = build_tenants_router(app_service).await;

  let auth_context = AuthContext::test_multi_tenant_session_full(
    "test-user",
    "test@example.com",
    "some-client",
    "some-tenant",
    services::ResourceRole::Admin,
    "some-token",
  );

  let resp = router
    .oneshot(
      Request::get(ENDPOINT_TENANTS)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: Value = resp.json().await?;
  let tenants = body["tenants"].as_array().unwrap();
  assert_eq!(1, tenants.len());
  assert_eq!("test-client-id", tenants[0]["client_id"].as_str().unwrap());
  assert_eq!("Test Tenant", tenants[0]["name"].as_str().unwrap());
  assert_eq!("ready", tenants[0]["status"].as_str().unwrap());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_create_succeeds_for_multi_tenant_session() -> anyhow::Result<()> {
  let mut mock_auth = MockAuthService::default();
  mock_auth.expect_create_tenant().returning(|_, _, _, _| {
    Ok(KcCreateTenantResponse {
      client_id: "new-tenant-client-id".to_string(),
      client_secret: "new-tenant-client-secret".to_string(),
    })
  });

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_multitenant_settings()
      .await
      .auth_service(Arc::new(mock_auth))
      .build()
      .await?,
  );
  let router = build_tenants_router(app_service).await;

  let auth_context = AuthContext::test_multi_tenant_session("test-user", "test@example.com");

  let resp = router
    .oneshot(
      Request::post(ENDPOINT_TENANTS)
        .json(json! {{"name": "New Tenant", "description": "A new tenant"}})?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, resp.status());
  let body: Value = resp.json().await?;
  assert_eq!("new-tenant-client-id", body["client_id"].as_str().unwrap());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_activate_returns_tenant_not_logged_in_for_multi_tenant_session(
) -> anyhow::Result<()> {
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_multitenant_settings()
      .await
      .build()
      .await?,
  );
  let router = build_tenants_router(app_service).await;

  let auth_context = AuthContext::test_multi_tenant_session("test-user", "test@example.com");

  let resp = router
    .oneshot(
      Request::post(&format!("{ENDPOINT_TENANTS}/some-client-id/activate"))
        .json(json! {{}})?
        .with_auth_context(auth_context),
    )
    .await?;

  let status = resp.status();
  assert_eq!(StatusCode::BAD_REQUEST, status);
  let body: Value = resp.json().await?;
  assert_eq!(
    "dashboard_auth_route_error-tenant_not_logged_in",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}
