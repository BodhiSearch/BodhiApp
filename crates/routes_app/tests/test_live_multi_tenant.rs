use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
};
use routes_app::{
  build_routes,
  test_utils::{AuthServerConfig, AuthServerTestClient},
  AppInfo, ENDPOINT_APP_INFO, ENDPOINT_DASHBOARD_AUTH_INITIATE, ENDPOINT_TENANTS,
  ENDPOINT_USER_INFO,
};
use rstest::{fixture, rstest};
use serde_json::Value;
use server_core::test_utils::ResponseTestExt;
use services::{
  test_utils::{AppServiceStubBuilder, SettingServiceStub},
  AppService, AppStatus, DefaultTenantService, KeycloakAuthService, MokaCacheService,
  TenantService, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_DEPLOYMENT, BODHI_HOST,
  BODHI_MULTITENANT_CLIENT_ID, BODHI_MULTITENANT_CLIENT_SECRET, BODHI_PORT, BODHI_SCHEME,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;
use tower_sessions::SessionStore;

// ── Env / Config ────────────────────────────────────────────────────────────

#[fixture]
#[once]
fn auth_server_config() -> AuthServerConfig {
  let env_path = PathBuf::from(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/resources/.env.test"
  ));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }

  AuthServerConfig {
    auth_server_url: std::env::var("INTEG_TEST_AUTH_URL").expect("INTEG_TEST_AUTH_URL must be set"),
    realm: std::env::var("INTEG_TEST_AUTH_REALM").expect("INTEG_TEST_AUTH_REALM must be set"),
    resource_client_id: std::env::var("INTEG_TEST_RESOURCE_CLIENT_ID")
      .expect("INTEG_TEST_RESOURCE_CLIENT_ID must be set"),
    resource_client_secret: std::env::var("INTEG_TEST_RESOURCE_CLIENT_SECRET")
      .expect("INTEG_TEST_RESOURCE_CLIENT_SECRET must be set"),
    app_client_id: std::env::var("INTEG_TEST_APP_CLIENT_ID")
      .expect("INTEG_TEST_APP_CLIENT_ID must be set"),
  }
}

struct MultiTenantConfig {
  client_id: String,
  client_secret: String,
}

fn multi_tenant_config() -> MultiTenantConfig {
  MultiTenantConfig {
    client_id: std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_ID")
      .expect("INTEG_TEST_MULTI_TENANT_CLIENT_ID must be set"),
    client_secret: std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET")
      .expect("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET must be set"),
  }
}

struct TestUser {
  username: String,
  password: String,
}

fn test_user() -> TestUser {
  TestUser {
    username: std::env::var("INTEG_TEST_USERNAME").expect("INTEG_TEST_USERNAME must be set"),
    password: std::env::var("INTEG_TEST_PASSWORD").expect("INTEG_TEST_PASSWORD must be set"),
  }
}

// ── App Service Builders ────────────────────────────────────────────────────

/// Build an AppService configured for multi-tenant mode with a real KeycloakAuthService.
async fn create_multi_tenant_state(
  config: &AuthServerConfig,
) -> anyhow::Result<Arc<dyn AppService>> {
  let mt = multi_tenant_config();

  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (BODHI_AUTH_URL.to_string(), config.auth_server_url.clone()),
      (BODHI_AUTH_REALM.to_string(), config.realm.clone()),
      (BODHI_DEPLOYMENT.to_string(), "multi_tenant".to_string()),
      (
        BODHI_MULTITENANT_CLIENT_ID.to_string(),
        mt.client_id.clone(),
      ),
      (
        BODHI_MULTITENANT_CLIENT_SECRET.to_string(),
        mt.client_secret.clone(),
      ),
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "localhost".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
    ]))
    .await;

  let auth_service = Arc::new(KeycloakAuthService::new(
    "test-app",
    config.auth_server_url.clone(),
    config.realm.clone(),
  ));

  let temp_dir = TempDir::new()?;
  let session_db_path = temp_dir.path().join("session.db");
  let shared_temp_dir = Arc::new(temp_dir);

  let mut builder = AppServiceStubBuilder::default();
  let test_db = services::test_utils::test_db_service_with_temp_dir(shared_temp_dir).await;
  let db_svc: Arc<dyn services::DbService> = Arc::new(test_db);
  let tenant_svc = DefaultTenantService::new(db_svc.clone());

  builder
    .setting_service(Arc::new(setting_service))
    .auth_service(auth_service)
    .db_service(db_svc)
    .tenant_service(Arc::new(tenant_svc) as Arc<dyn TenantService>)
    .cache_service(Arc::new(MokaCacheService::default()))
    .build_session_service(session_db_path)
    .await;

  let app_service = builder.build().await?;
  Ok(Arc::new(app_service) as Arc<dyn AppService>)
}

/// Build an AppService configured for standalone mode.
async fn create_standalone_state(config: &AuthServerConfig) -> anyhow::Result<Arc<dyn AppService>> {
  let setting_service = SettingServiceStub::default()
    .append_settings(HashMap::from([
      (BODHI_AUTH_URL.to_string(), config.auth_server_url.clone()),
      (BODHI_AUTH_REALM.to_string(), config.realm.clone()),
      (BODHI_DEPLOYMENT.to_string(), "standalone".to_string()),
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "localhost".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
    ]))
    .await;

  let auth_service = Arc::new(KeycloakAuthService::new(
    "test-app",
    config.auth_server_url.clone(),
    config.realm.clone(),
  ));

  let temp_dir = TempDir::new()?;
  let session_db_path = temp_dir.path().join("session.db");
  let shared_temp_dir = Arc::new(temp_dir);

  let mut builder = AppServiceStubBuilder::default();
  let test_db = services::test_utils::test_db_service_with_temp_dir(shared_temp_dir).await;
  let db_svc: Arc<dyn services::DbService> = Arc::new(test_db);

  builder
    .setting_service(Arc::new(setting_service))
    .auth_service(auth_service)
    .db_service(db_svc)
    .cache_service(Arc::new(MokaCacheService::default()))
    .build_session_service(session_db_path)
    .await;

  let app_service = builder.build().await?;
  Ok(Arc::new(app_service) as Arc<dyn AppService>)
}

// ── Session Helpers ─────────────────────────────────────────────────────────

/// Create a session record with the given data, save it, and return the session ID string.
async fn inject_session(
  app_service: &Arc<dyn AppService>,
  data: HashMap<String, Value>,
) -> anyhow::Result<String> {
  let session_service = app_service.session_service();
  let id = tower_sessions::session::Id::default();
  let mut record = tower_sessions::session::Record {
    id,
    data,
    expiry_date: time::OffsetDateTime::now_utc() + time::Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;
  Ok(format!("{}", record.id))
}

/// Get a dashboard token by doing a password grant against the multi-tenant client.
async fn get_dashboard_token(config: &AuthServerConfig) -> anyhow::Result<String> {
  let mt = multi_tenant_config();
  let user = test_user();
  let auth_client = AuthServerTestClient::new(config.clone());
  auth_client
    .get_user_token(
      &mt.client_id,
      &mt.client_secret,
      &user.username,
      &user.password,
      &["openid", "email", "profile"],
    )
    .await
}

/// Get a resource token by doing a password grant against a resource client.
async fn get_resource_token(
  config: &AuthServerConfig,
  client_id: &str,
  client_secret: &str,
) -> anyhow::Result<String> {
  let user = test_user();
  let auth_client = AuthServerTestClient::new(config.clone());
  auth_client
    .get_user_token(
      client_id,
      client_secret,
      &user.username,
      &user.password,
      &["openid", "email", "profile", "roles"],
    )
    .await
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// GET /bodhi/v1/info without any session in multi-tenant mode
/// should return status=tenant_selection and deployment=multi-tenant
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_info_multi_tenant_no_session(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;
  let router = build_routes(state, None).await;

  let request = Request::builder()
    .method("GET")
    .uri(ENDPOINT_APP_INFO)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .body(Body::empty())?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body: AppInfo = response.json().await?;
  assert_eq!(AppStatus::TenantSelection, body.status);
  assert_eq!(services::DeploymentMode::MultiTenant, body.deployment);
  assert_eq!(None, body.client_id);

  Ok(())
}

/// GET /bodhi/v1/info with dashboard + active tenant session in multi-tenant mode
/// should return status=ready with the active tenant's client_id
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_info_multi_tenant_with_dashboard_and_active_tenant(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;

  // Get a real dashboard token and a resource token for the resource client
  let dashboard_token = get_dashboard_token(auth_server_config).await?;
  let resource_token = get_resource_token(
    auth_server_config,
    &auth_server_config.resource_client_id,
    &auth_server_config.resource_client_secret,
  )
  .await?;

  let active_client_id = auth_server_config.resource_client_id.clone();

  // Register the resource client as a local tenant so the middleware can resolve it
  state
    .tenant_service()
    .create_tenant(
      &active_client_id,
      &auth_server_config.resource_client_secret,
      "Test Resource Tenant",
      None,
      services::AppStatus::Ready,
      Some("integration-test-user".to_string()),
    )
    .await?;

  // Inject session with dashboard token + active tenant + resource token
  let session_id = inject_session(
    &state,
    maplit::hashmap! {
      "dashboard:access_token".to_string() => Value::String(dashboard_token),
      "active_client_id".to_string() => Value::String(active_client_id.clone()),
      format!("{}:access_token", active_client_id) => Value::String(resource_token),
    },
  )
  .await?;

  let router = build_routes(state, None).await;

  let request = Request::builder()
    .method("GET")
    .uri(ENDPOINT_APP_INFO)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .body(Body::empty())?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body: AppInfo = response.json().await?;
  assert_eq!(AppStatus::Ready, body.status);
  assert_eq!(services::DeploymentMode::MultiTenant, body.deployment);
  assert_eq!(Some(active_client_id), body.client_id);

  Ok(())
}

/// POST /bodhi/v1/auth/dashboard/initiate in standalone mode
/// should return error with NotMultiTenant code
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_dashboard_auth_initiate_standalone_rejected(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_standalone_state(auth_server_config).await?;
  let router = build_routes(state, None).await;

  let request = Request::builder()
    .method("POST")
    .uri(ENDPOINT_DASHBOARD_AUTH_INITIATE)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Content-Type", "application/json")
    .body(Body::from("{}"))?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());

  let body: Value = response.json().await?;
  assert_eq!(
    "dashboard_auth_route_error-not_multi_tenant",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

/// POST /bodhi/v1/tenants/{client_id}/activate with a valid resource token in session
/// should return 200
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_activate_success(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;

  let active_client_id = auth_server_config.resource_client_id.clone();

  // Get a resource token for the tenant
  let resource_token = get_resource_token(
    auth_server_config,
    &auth_server_config.resource_client_id,
    &auth_server_config.resource_client_secret,
  )
  .await?;

  // Inject session with the resource token for this tenant
  let session_id = inject_session(
    &state,
    maplit::hashmap! {
      format!("{}:access_token", active_client_id) => Value::String(resource_token),
    },
  )
  .await?;

  let router = build_routes(state, None).await;

  let activate_url = format!("{}/{}/activate", ENDPOINT_TENANTS, active_client_id);
  let request = Request::builder()
    .method("POST")
    .uri(&activate_url)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .header("Content-Type", "application/json")
    .body(Body::from("{}"))?;

  let response = router.oneshot(request).await?;
  assert_eq!(
    StatusCode::OK,
    response.status(),
    "Activate failed: {}",
    response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );

  Ok(())
}

/// POST /bodhi/v1/tenants/{client_id}/activate without a resource token
/// should return TenantNotLoggedIn error
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tenants_activate_not_logged_in(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;

  // Inject an empty session (no resource token for any tenant)
  let session_id = inject_session(&state, maplit::hashmap! {}).await?;

  let router = build_routes(state, None).await;

  let activate_url = format!("{}/some-nonexistent-client/activate", ENDPOINT_TENANTS);
  let request = Request::builder()
    .method("POST")
    .uri(&activate_url)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .header("Content-Type", "application/json")
    .body(Body::from("{}"))?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body: Value = response.json().await?;
  assert_eq!(
    "dashboard_auth_route_error-tenant_not_logged_in",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

/// GET /bodhi/v1/user with a dashboard token in session
/// should return dashboard object and auth_status: logged_out (no resource token)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_has_dashboard_session(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;

  let dashboard_token = get_dashboard_token(auth_server_config).await?;

  let session_id = inject_session(
    &state,
    maplit::hashmap! {
      "dashboard:access_token".to_string() => Value::String(dashboard_token),
    },
  )
  .await?;

  let router = build_routes(state, None).await;

  let request = Request::builder()
    .method("GET")
    .uri(ENDPOINT_USER_INFO)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .body(Body::empty())?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body: Value = response.json().await?;
  assert_eq!("logged_out", body["auth_status"].as_str().unwrap());
  assert!(
    body["dashboard"].is_object(),
    "Expected dashboard to be an object, got: {:?}",
    body.get("dashboard")
  );
  assert!(body["dashboard"]["user_id"].is_string());
  assert!(body["dashboard"]["username"].is_string());
  assert!(body.get("has_dashboard_session").is_none());

  Ok(())
}

/// GET /bodhi/v1/user without a dashboard token in session
/// should NOT include dashboard field
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_no_dashboard_session(
  auth_server_config: &AuthServerConfig,
) -> anyhow::Result<()> {
  let state = create_multi_tenant_state(auth_server_config).await?;
  let router = build_routes(state, None).await;

  let request = Request::builder()
    .method("GET")
    .uri(ENDPOINT_USER_INFO)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .body(Body::empty())?;

  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());

  let body: Value = response.json().await?;
  assert!(
    body.get("dashboard").is_none(),
    "Expected dashboard to be absent, got: {:?}",
    body.get("dashboard")
  );
  assert_eq!("logged_out", body["auth_status"].as_str().unwrap());

  Ok(())
}
