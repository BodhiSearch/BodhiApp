use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  extract::State,
  http::{Request, StatusCode},
  middleware::from_fn_with_state,
  response::Json,
  routing::get,
  Extension, Router,
};
use routes_app::{
  middleware::auth_middleware,
  test_utils::{AuthServerConfig, AuthServerTestClient, TestUser},
};
use rstest::{fixture, rstest};
use server_core::test_utils::ResponseTestExt;
#[allow(unused_imports)]
use services::AccessRequestRepository;
use services::AuthContext;
use services::{
  extract_claims,
  test_utils::{
    test_db_service_with_temp_dir, AppServiceStubBuilder, SettingServiceStub, TEST_TENANT_ID,
  },
  AppService, AppStatus, Claims, DefaultTenantService, KeycloakAuthService, TenantService,
  BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TestTokenResponse {
  token: Option<String>,
  role: Option<String>,
}

async fn test_token_info_handler(
  auth_context: Option<Extension<AuthContext>>,
  State(_app_service): State<Arc<dyn AppService>>,
) -> Json<TestTokenResponse> {
  let auth_context = auth_context.map(|Extension(ctx)| ctx);
  let token = auth_context
    .as_ref()
    .and_then(|ctx| ctx.token())
    .map(|s| s.to_string());
  let role = auth_context.as_ref().and_then(|ctx| match ctx {
    AuthContext::ApiToken { role, .. } => Some(format!("{}", role)),
    AuthContext::ExternalApp { role, .. } => role.as_ref().map(|r| format!("{}", r)),
    _ => None,
  });
  Json(TestTokenResponse { token, role })
}

fn create_test_router(app_service: Arc<dyn AppService>) -> Router {
  Router::new()
    .merge(
      Router::new()
        .route("/test", get(test_token_info_handler))
        .route_layer(from_fn_with_state(app_service.clone(), auth_middleware)),
    )
    .layer(app_service.session_service().session_layer(false))
    .with_state(app_service)
}

#[fixture]
fn auth_client(auth_server_config: &AuthServerConfig) -> AuthServerTestClient {
  AuthServerTestClient::new(auth_server_config.clone())
}

async fn create_test_state(config: &AuthServerConfig) -> anyhow::Result<Arc<dyn AppService>> {
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_AUTH_URL.to_string(), config.auth_server_url.clone()),
    (BODHI_AUTH_REALM.to_string(), config.realm.clone()),
  ]));

  let auth_service = Arc::new(KeycloakAuthService::new(
    "test-app",
    config.auth_server_url.clone(),
    config.realm.clone(),
  ));

  let temp_dir = TempDir::new()?;
  let session_db_path = temp_dir.path().join("session.db");
  let shared_temp_dir = Arc::new(temp_dir);

  let mut app_service_builder = AppServiceStubBuilder::default();
  let test_db = test_db_service_with_temp_dir(shared_temp_dir).await;
  let db_svc: Arc<dyn services::DbService> = Arc::new(test_db);
  let tenant_svc = DefaultTenantService::new(db_svc.clone());
  tenant_svc
    .create_tenant(
      &config.resource_client_id,
      &config.resource_client_secret,
      "Test App",
      None,
      AppStatus::Ready,
      Some("integration-test-user".to_string()),
    )
    .await?;

  app_service_builder
    .tenant_service(Arc::new(tenant_svc) as Arc<dyn TenantService>)
    .setting_service(Arc::new(setting_service))
    .auth_service(auth_service)
    .db_service(db_svc)
    .cache_service(Arc::new(services::MokaCacheService::default()))
    .build_session_service(session_db_path)
    .await;

  let app_service = app_service_builder.build().await?;
  Ok(Arc::new(app_service) as Arc<dyn AppService>)
}

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

#[fixture]
fn test_user() -> TestUser {
  TestUser {
    username: std::env::var("INTEG_TEST_USERNAME").expect("INTEG_TEST_USERNAME must be set"),
    user_id: std::env::var("INTEG_TEST_USERNAME_ID").expect("INTEG_TEST_USERNAME_ID must be set"),
    password: std::env::var("INTEG_TEST_PASSWORD").expect("INTEG_TEST_PASSWORD must be set"),
  }
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_cross_client_token_exchange_success(
  auth_server_config: &AuthServerConfig,
  test_user: TestUser,
  auth_client: AuthServerTestClient,
) -> anyhow::Result<()> {
  let state = create_test_state(auth_server_config).await?;

  let db_service = state.db_service();
  let actual_tenant_id = state
    .tenant_service()
    .get_standalone_app()
    .await?
    .map(|t| t.id)
    .unwrap_or_else(|| TEST_TENANT_ID.to_string());
  let access_request_id = uuid::Uuid::new_v4().to_string();

  // Stateless KC mapper contract: the dotted scope is `<resource-client-id>.<row-id>`
  // (split on the last dot), composed locally — no consent registration call anymore.
  let access_request_scope = format!(
    "{}{}.{}",
    services::SCOPE_ACCESS_REQUEST_PREFIX,
    auth_server_config.resource_client_id,
    access_request_id
  );
  let now = chrono::Utc::now();
  let row = services::AppAccessRequest {
    app_client_id: auth_server_config.app_client_id.clone(),
    access_request_scope: Some(access_request_scope.clone()),
    ..services::test_utils::approved_request(
      &access_request_id,
      &actual_tenant_id,
      &test_user.user_id,
      now,
    )
  };
  db_service.create(&row).await?;

  // Get bearer token WITH scope_access_request:<client>.<uuid> — KC injects aud and
  // access_request_id claim via the stateless mapper
  let scopes = vec![
    "openid",
    "email",
    "profile",
    "roles",
    access_request_scope.as_str(),
  ];
  let user_token = auth_client
    .get_app_user_token_with_scope(
      &auth_server_config.app_client_id,
      &test_user.username,
      &test_user.password,
      &scopes,
    )
    .await?;

  let router = create_test_router(state);
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(Body::empty())?;
  let response = router.oneshot(request).await?;

  assert_eq!(
    StatusCode::OK,
    response.status(),
    "Token exchange failed: {}",
    response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );

  let body: TestTokenResponse = response.json().await?;
  assert!(body.token.is_some(), "Expected token to be set");

  let token = body.token.as_ref().unwrap();
  let claims = extract_claims::<Claims>(token)?;
  assert_eq!(
    claims.preferred_username, test_user.username,
    "JWT preferred_username claim should match test user"
  );
  assert_eq!(claims.azp, auth_server_config.resource_client_id);
  assert_eq!(
    Some("scope_user_user".to_string()),
    body.role,
    "Expected role scope_user_user from approved access request"
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_cross_client_token_exchange_auth_service_error(
  auth_server_config: &AuthServerConfig,
  test_user: TestUser,
  auth_client: AuthServerTestClient,
) -> anyhow::Result<()> {
  let state = create_test_state(auth_server_config).await?;

  // Get token WITHOUT scope_access_request:* → KC does NOT inject aud → audience check fails
  let user_token = auth_client
    .get_app_user_token_with_scope(
      &auth_server_config.app_client_id,
      &test_user.username,
      &test_user.password,
      &["openid", "email", "profile", "roles"],
    )
    .await?;

  let router = create_test_router(state);
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(Body::empty())?;
  let response = router.oneshot(request).await?;

  assert_eq!(StatusCode::UNAUTHORIZED, response.status());

  let body = response.text().await?;
  assert!(
    body.contains("audience") || body.contains("aud"),
    "Expected aud-related error, got: {}",
    body
  );

  Ok(())
}
