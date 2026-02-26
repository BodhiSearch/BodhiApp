use anyhow_trace::anyhow_trace;
use auth_middleware::{
  auth_middleware,
  test_utils::{AuthServerConfig, AuthServerTestClient, TestUser},
  AuthContext,
};
use axum::{
  body::Body,
  extract::State,
  http::{Request, StatusCode},
  middleware::from_fn_with_state,
  response::Json,
  routing::get,
  Extension, Router,
};
use rstest::{fixture, rstest};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  extract_claims,
  test_utils::{test_db_service_with_temp_dir, AppServiceStubBuilder, SettingServiceStub},
  AppInstanceService, AppStatus, Claims, DefaultAppInstanceService, KeycloakAuthService,
  BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

// Test response structure for our test endpoint
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TestTokenResponse {
  token: Option<String>,
  role: Option<String>,
}

// Test endpoint that returns info about injected auth context
async fn test_token_info_handler(
  auth_context: Option<Extension<AuthContext>>,
  State(_state): State<Arc<dyn RouterState>>,
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

async fn create_test_state(config: &AuthServerConfig) -> anyhow::Result<Arc<DefaultRouterState>> {
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
  let db_svc: Arc<dyn services::db::DbService> = Arc::new(test_db);
  let app_instance_svc = DefaultAppInstanceService::new(db_svc.clone());
  app_instance_svc
    .create_instance(
      &config.resource_client_id,
      &config.resource_client_secret,
      AppStatus::Ready,
    )
    .await?;

  app_service_builder
    .app_instance_service(Arc::new(app_instance_svc) as Arc<dyn AppInstanceService>)
    .setting_service(Arc::new(setting_service))
    .auth_service(auth_service)
    .db_service(db_svc)
    .cache_service(Arc::new(services::MokaCacheService::default()))
    .build_session_service(session_db_path)
    .await;

  let app_service = app_service_builder.build().await?;
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

  // Create a draft access request in DB
  let db_service = state.app_service().db_service();
  let access_request_id = uuid::Uuid::new_v4().to_string();
  let now = chrono::Utc::now().timestamp();
  let expires_at = now + 600;
  let row = services::db::AppAccessRequestRow {
    id: access_request_id.clone(),
    app_client_id: auth_server_config.app_client_id.clone(),
    app_name: None,
    app_description: None,
    flow_type: "popup".to_string(),
    redirect_uri: None,
    status: "draft".to_string(),
    requested: r#"{"toolset_types":[],"mcp_servers":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
  };
  db_service.create(&row).await?;

  // Register the consent with Keycloak using a token from the resource client
  // KC requires the token to be from the resource client (not the app client)
  let resource_user_token = auth_client
    .get_user_token(
      &auth_server_config.resource_client_id,
      &auth_server_config.resource_client_secret,
      &test_user.username,
      &test_user.password,
      &["openid", "email", "profile", "roles"],
    )
    .await?;

  let auth_service = state.app_service().auth_service();
  let kc_response = auth_service
    .register_access_request_consent(
      &resource_user_token,
      &auth_server_config.app_client_id,
      &access_request_id,
      "Access approved",
    )
    .await?;

  // Approve the access request in DB using the KC-returned scope
  let access_request_scope = kc_response.access_request_scope;
  let approved_json = r#"{"toolsets":[],"mcps":[]}"#;
  db_service
    .update_approval(
      &access_request_id,
      &test_user.user_id,
      approved_json,
      "scope_user_user",
      &access_request_scope,
    )
    .await?;

  // Get bearer token WITH scope_access_request:<uuid> — KC injects aud and access_request_id claim
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
  // Verify the error is aud-related (missing audience or invalid audience)
  assert!(
    body.contains("audience") || body.contains("aud"),
    "Expected aud-related error, got: {}",
    body
  );

  Ok(())
}
