use crate::setup::routes_setup::{setup_create, setup_show};
use crate::setup::setup_api_schemas::{AppInfo, SetupRequest};
use crate::test_utils::{RequestAuthContextExt, TEST_ENDPOINT_APP_INFO};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{get, post},
  Router,
};

use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::ReqwestError;
use services::{
  test_utils::{AppServiceStubBuilder, SettingServiceStub, TEST_TENANT_ID, TEST_USER_ID},
  AppService, AppStatus, AuthContext, AuthServiceError, ClientRegistrationResponse, DeploymentMode,
  MockAuthService, Tenant, BODHI_DEPLOYMENT,
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[anyhow_trace]
#[rstest]
#[case(
  AppStatus::Setup,
  AppInfo {
    version: "0.0.0".to_string(),
    commit_sha: "test-sha".to_string(),
    status: AppStatus::Setup,
    deployment: services::DeploymentMode::Standalone,
    client_id: Some("test-client".to_string()),
    url: "http://localhost:1135".to_string(),
  }
)]
#[tokio::test]
async fn test_app_info_handler(
  #[case] status: AppStatus,
  #[case] expected: AppInfo,
) -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_tenant(Tenant::test_with_status(status)).await;
  builder.with_session_service().await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn services::AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);
  let resp = router
    .oneshot(Request::get(TEST_ENDPOINT_APP_INFO).body(Body::empty())?)
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(expected, value);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_handler_with_client_id() -> anyhow::Result<()> {
  use services::test_utils::TEST_TENANT_ID;
  use services::AuthContext;

  let mut builder = AppServiceStubBuilder::default();
  builder
    .with_tenant(Tenant::test_with_status(AppStatus::Ready))
    .await;
  builder.with_session_service().await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn services::AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let auth_context = AuthContext::Session {
    client_id: "my-test-client-id".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    username: "testuser".to_string(),
    role: services::ResourceRole::Guest,
    token: "dummy-token".to_string(),
  };

  let resp = router
    .oneshot(
      Request::get(TEST_ENDPOINT_APP_INFO)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(
    AppInfo {
      version: "0.0.0".to_string(),
      commit_sha: "test-sha".to_string(),
      status: AppStatus::Ready,
      deployment: services::DeploymentMode::Standalone,
      client_id: Some("my-test-client-id".to_string()),
      url: "http://localhost:1135".to_string(),
    },
    value
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_error() -> anyhow::Result<()> {
  let payload = SetupRequest {
    name: "Test Server Name".to_string(),
    description: Some("Test description".to_string()),
  };
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant(Tenant::test_default())
      .await
      .auth_service(Arc::new(MockAuthService::new()))
      .build()
      .await?,
  );
  let state = app_service.clone();

  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(payload)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!(
    "setup_route_error-already_setup",
    body["error"]["code"].as_str().unwrap()
  );

  let tenant_service = app_service.tenant_service();
  let status = tenant_service
    .get_standalone_app()
    .await?
    .map(|t| t.status)
    .unwrap_or_default();
  assert_eq!(AppStatus::Ready, status);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_success() -> anyhow::Result<()> {
  let request = SetupRequest {
    name: "Test Server Name".to_string(),
    description: Some("Test description".to_string()),
  };
  let expected_status = AppStatus::ResourceAdmin;
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_register_client()
    .times(1)
    .return_once(|_name, _description, _redirect_uris| {
      Ok(ClientRegistrationResponse {
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
      })
    });
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .build()
      .await?,
  );
  let state = app_service.clone();
  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let response = router
    .oneshot(Request::post("/setup").json(request)?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let tenant_service = app_service.tenant_service();
  let status = tenant_service
    .get_standalone_app()
    .await?
    .map(|t| t.status)
    .unwrap_or_default();
  assert_eq!(expected_status, status);
  let instance = tenant_service.get_standalone_app().await?;
  assert!(instance.is_some());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_loopback_redirect_uris() -> anyhow::Result<()> {
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_register_client()
    .times(1)
    .withf(|_name, _description, redirect_uris| {
      // Verify that all loopback redirect URIs are registered
      // Now there might be additional URIs (request host, server IP) so check >= 3
      redirect_uris.len() >= 3
        && redirect_uris.contains(&"http://localhost:1135/ui/auth/callback".to_string())
        && redirect_uris.contains(&"http://127.0.0.1:1135/ui/auth/callback".to_string())
        && redirect_uris.contains(&"http://0.0.0.0:1135/ui/auth/callback".to_string())
    })
    .return_once(|_name, _description, _redirect_uris| {
      Ok(ClientRegistrationResponse {
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
      })
    });

  // Configure with default settings (no explicit public host)
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (services::BODHI_SCHEME.to_string(), "http".to_string()),
      (services::BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (services::BODHI_PORT.to_string(), "1135".to_string()),
    ]))
    .await;

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .setting_service(Arc::new(setting_service))
      .build()
      .await?,
  );
  let state = app_service.clone();

  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let request = SetupRequest {
    name: "Test Server Name".to_string(),
    description: Some("Test description".to_string()),
  };

  let response = router
    .oneshot(
      Request::post("/setup")
        .header("Host", "localhost:1135")
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let tenant_service = app_service.tenant_service();
  let status = tenant_service
    .get_standalone_app()
    .await?
    .map(|t| t.status)
    .unwrap_or_default();
  assert_eq!(AppStatus::ResourceAdmin, status);
  let instance = tenant_service.get_standalone_app().await?;
  assert!(instance.is_some());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_network_ip_redirect_uris() -> anyhow::Result<()> {
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_register_client()
    .times(1)
    .withf(|_name, _description, redirect_uris| {
      // Verify that all loopback hosts AND the network IP are registered
      redirect_uris.len() >= 4  // 3 loopback + 1 network IP (+ optional server IP)
        && redirect_uris.contains(&"http://localhost:1135/ui/auth/callback".to_string())
        && redirect_uris.contains(&"http://127.0.0.1:1135/ui/auth/callback".to_string())
        && redirect_uris.contains(&"http://0.0.0.0:1135/ui/auth/callback".to_string())
        && redirect_uris.contains(&"http://192.168.1.100:1135/ui/auth/callback".to_string())
    })
    .return_once(|_name, _description, _redirect_uris| {
      Ok(ClientRegistrationResponse {
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
      })
    });

  // Configure with default settings (no explicit public host)
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (services::BODHI_SCHEME.to_string(), "http".to_string()),
      (services::BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (services::BODHI_PORT.to_string(), "1135".to_string()),
    ]))
    .await;

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .setting_service(Arc::new(setting_service))
      .build()
      .await?,
  );
  let state = app_service.clone();

  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let request = SetupRequest {
    name: "Test Server Name".to_string(),
    description: Some("Test description".to_string()),
  };

  let response = router
    .oneshot(
      Request::post("/setup")
        .header("Host", "192.168.1.100:1135")
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let tenant_service = app_service.tenant_service();
  let status = tenant_service
    .get_standalone_app()
    .await?
    .map(|t| t.status)
    .unwrap_or_default();
  assert_eq!(AppStatus::ResourceAdmin, status);
  let instance = tenant_service.get_standalone_app().await?;
  assert!(instance.is_some());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_explicit_public_host_single_redirect_uri() -> anyhow::Result<()> {
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_register_client()
    .times(1)
    .withf(|_name, _description, redirect_uris| {
      // When public host is explicitly set, should only register that one
      redirect_uris.len() == 1
        && redirect_uris.contains(&"https://my-bodhi.example.com:8443/ui/auth/callback".to_string())
    })
    .return_once(|_name, _description, _redirect_uris| {
      Ok(ClientRegistrationResponse {
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
      })
    });

  // Configure with explicit public host
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (
        services::BODHI_PUBLIC_SCHEME.to_string(),
        "https".to_string(),
      ),
      (
        services::BODHI_PUBLIC_HOST.to_string(),
        "my-bodhi.example.com".to_string(),
      ),
      (services::BODHI_PUBLIC_PORT.to_string(), "8443".to_string()),
    ]))
    .await;

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .setting_service(Arc::new(setting_service))
      .build()
      .await?,
  );
  let state = app_service.clone();

  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let request = SetupRequest {
    name: "Test Server Name".to_string(),
    description: Some("Test description".to_string()),
  };

  let response = router
    .oneshot(
      Request::post("/setup")
        .header("Host", "192.168.1.100:1135") // This should be ignored due to explicit config
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let tenant_service = app_service.tenant_service();
  let status = tenant_service
    .get_standalone_app()
    .await?
    .map(|t| t.status)
    .unwrap_or_default();
  assert_eq!(AppStatus::ResourceAdmin, status);
  let instance = tenant_service.get_standalone_app().await?;
  assert!(instance.is_some());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_register_resource_error() -> anyhow::Result<()> {
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_register_client()
    .times(1)
    .return_once(|_name, _description, _redirect_uris| {
      Err(AuthServiceError::Reqwest(ReqwestError::new(
        "failed to register as resource server".to_string(),
      )))
    });
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .build()
      .await?,
  );
  let state = app_service.clone();
  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(SetupRequest {
      name: "Test Server Name".to_string(),
      description: Some("Test description".to_string()),
    })?)
    .await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!("reqwest_error", body["error"]["code"].as_str().unwrap());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case(r#"{"invalid": true,}"#)]
#[tokio::test]
async fn test_setup_handler_bad_request(#[case] body: &str) -> anyhow::Result<()> {
  let app_service = Arc::new(AppServiceStubBuilder::default().build().await?);
  let state = app_service.clone();
  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json_str(body)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!(
    "json_rejection_error",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_validation_error() -> anyhow::Result<()> {
  let mock_auth_service = MockAuthService::default();
  // No expectation needed as validation should fail before calling auth service

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_tenant_service()
      .await
      .auth_service(Arc::new(mock_auth_service))
      .build()
      .await?,
  );
  let state = app_service.clone();
  let router = Router::new()
    .route("/setup", post(setup_create))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(SetupRequest {
      name: "Short".to_string(), // Less than 10 characters
      description: Some("Test description".to_string()),
    })?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  Ok(())
}

// ── Multi-tenant setup_show() tests ─────────────────────────────────────────

use services::test_utils::AppServiceStub;

async fn build_multi_tenant_app_service() -> anyhow::Result<Arc<AppServiceStub>> {
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([(
      BODHI_DEPLOYMENT.to_string(),
      "multi_tenant".to_string(),
    )]))
    .await;
  let mut builder = AppServiceStubBuilder::default();
  builder
    .with_tenant_service()
    .await
    .setting_service(Arc::new(setting_service));
  Ok(Arc::new(builder.build().await?))
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_multi_tenant_anonymous_returns_ready() -> anyhow::Result<()> {
  let app_service = build_multi_tenant_app_service().await?;
  let state: Arc<dyn AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let auth_context = AuthContext::test_anonymous(DeploymentMode::MultiTenant);
  let resp = router
    .oneshot(
      Request::get(TEST_ENDPOINT_APP_INFO)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(AppStatus::Ready, value.status);
  assert_eq!(None, value.client_id);
  assert_eq!(DeploymentMode::MultiTenant, value.deployment);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_multi_tenant_dashboard_with_memberships_returns_ready() -> anyhow::Result<()>
{
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([(
      BODHI_DEPLOYMENT.to_string(),
      "multi_tenant".to_string(),
    )]))
    .await;
  let tenant = Tenant::test_default();
  let mut builder = AppServiceStubBuilder::default();
  builder
    .with_tenant(tenant)
    .await
    .setting_service(Arc::new(setting_service));
  let app_service = Arc::new(builder.build().await?);

  // Create a tenant-user membership so has_memberships returns true
  app_service
    .tenant_service()
    .upsert_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  let state: Arc<dyn AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let auth_context = AuthContext::test_multi_tenant_session(TEST_USER_ID, "testuser");
  let resp = router
    .oneshot(
      Request::get(TEST_ENDPOINT_APP_INFO)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(AppStatus::Ready, value.status);
  assert_eq!(None, value.client_id);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_multi_tenant_dashboard_without_memberships_returns_setup(
) -> anyhow::Result<()> {
  let app_service = build_multi_tenant_app_service().await?;
  let state: Arc<dyn AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let auth_context = AuthContext::test_multi_tenant_session(TEST_USER_ID, "testuser");
  let resp = router
    .oneshot(
      Request::get(TEST_ENDPOINT_APP_INFO)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(AppStatus::Setup, value.status);
  assert_eq!(None, value.client_id);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_multi_tenant_session_with_client_id_returns_ready() -> anyhow::Result<()> {
  let app_service = build_multi_tenant_app_service().await?;
  let state: Arc<dyn AppService> = app_service.clone();
  let router = Router::new()
    .route(TEST_ENDPOINT_APP_INFO, get(setup_show))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let auth_context = AuthContext::test_multi_tenant_session_full(
    TEST_USER_ID,
    "testuser",
    "test-client",
    TEST_TENANT_ID,
    services::ResourceRole::Admin,
    "test-token",
  );
  let resp = router
    .oneshot(
      Request::get(TEST_ENDPOINT_APP_INFO)
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let value = resp.json::<AppInfo>().await?;
  assert_eq!(AppStatus::Ready, value.status);
  assert_eq!(Some("test-client".to_string()), value.client_id);
  Ok(())
}
