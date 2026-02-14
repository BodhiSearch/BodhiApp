use anyhow_trace::anyhow_trace;
use auth_middleware::{
  auth_middleware,
  test_utils::{AuthServerConfig, AuthServerTestClient, TestUser},
  KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
};
use axum::{
  body::Body,
  extract::State,
  http::{HeaderMap, Request, StatusCode},
  middleware::from_fn_with_state,
  response::Json,
  routing::get,
  Router,
};
use objs::{ErrorBody, OpenAIApiError};
use rstest::{fixture, rstest};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  extract_claims,
  test_utils::{test_db_service, AppServiceStubBuilder, SecretServiceStub, SettingServiceStub},
  AppRegInfoBuilder, Claims, KeycloakAuthService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use std::{collections::HashMap, env, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

// Test response structure for our test endpoint
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TestTokenResponse {
  token: Option<String>,
  scope: Option<String>,
}

// Test endpoint that returns info about injected auth headers
async fn test_token_info_handler(
  headers: HeaderMap,
  State(_state): State<Arc<dyn RouterState>>,
) -> Json<TestTokenResponse> {
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .and_then(|t| t.to_str().ok())
    .map(|s| s.to_string());
  let scope = headers
    .get(KEY_HEADER_BODHIAPP_SCOPE)
    .and_then(|s| s.to_str().ok())
    .map(|s| s.to_string());
  Json(TestTokenResponse { token, scope })
}

fn create_test_router(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .merge(
      Router::new()
        .route("/test", get(test_token_info_handler))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware)),
    )
    .layer(state.app_service().session_service().session_layer())
    .with_state(state)
}

#[fixture]
fn auth_client(auth_server_config: &AuthServerConfig) -> AuthServerTestClient {
  AuthServerTestClient::new(auth_server_config.clone())
}

// Helper function to create test state with specific client configuration
async fn create_test_state(
  config: &AuthServerConfig,
  resource_client_id: &str,
  resource_client_secret: &str,
) -> anyhow::Result<Arc<DefaultRouterState>> {
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
  let secret_service = SecretServiceStub::default().with_app_reg_info(
    &AppRegInfoBuilder::default()
      .client_id(resource_client_id.to_string())
      .client_secret(resource_client_secret.to_string())
      .build()
      .unwrap(),
  );

  let mut app_service_builder = AppServiceStubBuilder::default();
  let test_db_service = test_db_service(temp_dir).await;
  app_service_builder
    .secret_service(Arc::new(secret_service))
    .setting_service(Arc::new(setting_service))
    .auth_service(auth_service)
    .db_service(Arc::new(test_db_service))
    .cache_service(Arc::new(services::MokaCacheService::default()))
    .build_session_service(session_db_path)
    .await;

  let app_service = app_service_builder.build()?;
  let state = DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  );
  Ok(Arc::new(state))
}

#[fixture]
#[once]
fn auth_server_config() -> AuthServerConfig {
  let env_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/.env.test"));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }

  AuthServerConfig {
    auth_server_url: std::env::var("INTEG_TEST_AUTH_URL").expect("INTEG_TEST_AUTH_URL must be set"),
    realm: std::env::var("INTEG_TEST_AUTH_REALM").expect("INTEG_TEST_AUTH_REALM must be set"),
    dev_console_client_id: std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_ID")
      .expect("INTEG_TEST_DEV_CONSOLE_CLIENT_ID must be set"),
    dev_console_client_secret: std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET")
      .expect("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET must be set"),
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
  // Step 1: Setup dynamic clients
  let dynamic_clients = auth_client.setup_dynamic_clients(&test_user).await?;

  // Step 2: Create test state with dynamic client credentials
  let resource_client_id = dynamic_clients.resource_client.client_id;
  let resource_client_secret = dynamic_clients
    .resource_client
    .client_secret
    .as_ref()
    .unwrap();
  let state = create_test_state(
    auth_server_config,
    &resource_client_id,
    resource_client_secret,
  )
  .await?;
  let user_token = auth_client
    .get_app_user_token_with_scope(
      &dynamic_clients.app_client.client_id,
      &test_user.username,
      &test_user.password,
      &[
        "openid",
        "email",
        "profile",
        "roles",
        "scope_user_user",
        &dynamic_clients.resource_scope_name,
      ],
    )
    .await?;
  let router = create_test_router(state);
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(Body::empty())?;
  let response = router.oneshot(request).await?;

  // Step 5: Verify successful token exchange
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
  assert!(
    body.token.is_some(),
    "Expected X-Resource-Token header to be set"
  );
  assert!(
    body.scope.is_some(),
    "Expected X-Resource-Scope header to be set"
  );

  // Step 6: Decode JWT and assert claims
  let token = body.token.as_ref().unwrap();
  let claims = extract_claims::<Claims>(token)?;
  assert_eq!(
    claims.preferred_username, test_user.username,
    "JWT preferred_username claim should match test user"
  );
  assert_eq!(claims.azp, resource_client_id);
  let mut scopes = claims.scope.split_whitespace().collect::<Vec<&str>>();
  scopes.sort();
  let mut expected_scope = vec!["email", "openid", "profile", "roles", "scope_user_user"];
  expected_scope.sort();
  assert_eq!(scopes, expected_scope, "JWT scope should match expected");
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_cross_client_token_exchange_no_user_scope(
  auth_server_config: &AuthServerConfig,
  test_user: TestUser,
  auth_client: AuthServerTestClient,
) -> anyhow::Result<()> {
  // Step 1: Setup dynamic clients
  let dynamic_clients = auth_client.setup_dynamic_clients(&test_user).await?;

  // Step 2: Create test state with dynamic client credentials
  let resource_client_id = dynamic_clients.resource_client.client_id;
  let resource_client_secret = dynamic_clients
    .resource_client
    .client_secret
    .as_ref()
    .unwrap();
  let state = create_test_state(
    auth_server_config,
    &resource_client_id,
    resource_client_secret,
  )
  .await?;
  let user_token = auth_client
    .get_app_user_token_with_scope(
      &dynamic_clients.app_client.client_id,
      &test_user.username,
      &test_user.password,
      &[
        "openid",
        "email",
        "profile",
        "roles",
        &dynamic_clients.resource_scope_name,
      ],
    )
    .await?;

  // Step 4: Test token exchange - should return unauthorized
  let router = create_test_router(state);
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(Body::empty())?;
  let response = router.oneshot(request).await?;

  // Step 5: Verify unauthorized response
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let err: OpenAIApiError = response.json().await?;
  assert_eq!(
    OpenAIApiError::new(
      ErrorBody::new(
        "User does not have any access permissions.".to_string(),
        "authentication_error".to_string(),
        Some("token_error-scope_empty".to_string()),
        None,
      ),
      0,
    ),
    err
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
  // Step 1: Setup dynamic clients
  let dynamic_clients = auth_client.setup_dynamic_clients(&test_user).await?;

  // Step 2: Create test state with dynamic client credentials
  let state = create_test_state(
    auth_server_config,
    &dynamic_clients.resource_client.client_id,
    dynamic_clients
      .resource_client
      .client_secret
      .as_ref()
      .unwrap(),
  )
  .await?;
  let user_token = auth_client
    .get_app_user_token_with_scope(
      &dynamic_clients.app_client.client_id,
      &test_user.username,
      &test_user.password,
      &[],
    )
    .await?;
  let router = create_test_router(state);
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(Body::empty())?;
  let response = router.oneshot(request).await?;
  assert!(
    response.status().is_client_error(),
    "Expected client error, got {}",
    response.status()
  );
  Ok(())
}
