use anyhow_trace::anyhow_trace;
use auth_middleware::{
  auth_middleware, KEY_RESOURCE_SCOPE, KEY_RESOURCE_TOKEN, SESSION_KEY_ACCESS_TOKEN,
};
use axum::{
  extract::State,
  http::{HeaderMap, Request, StatusCode},
  middleware::from_fn_with_state,
  response::Json,
  routing::{get, post},
  Router,
};
use dotenv;
use maplit::hashmap;
use objs::{test_utils::setup_l10n, ErrorBody, FluentLocalizationService, OpenAIApiError};
use reqwest::Client;
use rstest::{fixture, rstest};
use serde_json::Value;
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  extract_claims,
  test_utils::{test_db_service, AppServiceStubBuilder, SecretServiceStub, SettingServiceStub},
  AppRegInfoBuilder, Claims, KeycloakAuthService, OfflineClaims, SecretServiceExt,
  BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use std::{collections::HashMap, env, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::{
  cookie::{Cookie, SameSite},
  session::{Id, Record},
  SessionStore,
};

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
    .get(KEY_RESOURCE_TOKEN)
    .and_then(|t| t.to_str().ok())
    .map(|s| s.to_string());
  let scope = headers
    .get(KEY_RESOURCE_SCOPE)
    .and_then(|s| s.to_str().ok())
    .map(|s| s.to_string());
  Json(TestTokenResponse { token, scope })
}

#[fixture]
#[awt]
async fn state(
  #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  integ_test_config: &IntegTestConfig,
) -> anyhow::Result<DefaultRouterState> {
  let setting_service = SettingServiceStub::new(HashMap::from([
    (
      BODHI_AUTH_URL.to_string(),
      integ_test_config.auth_server_url.clone(),
    ),
    (
      BODHI_AUTH_REALM.to_string(),
      integ_test_config.realm.clone(),
    ),
  ]));
  let auth_service = Arc::new(KeycloakAuthService::new(
    "test-app",
    integ_test_config.auth_server_url.clone(),
    integ_test_config.realm.clone(),
  ));
  let temp_dir = TempDir::new()?;
  let session_db_path = temp_dir.path().join("session.db");
  let secret_service = SecretServiceStub::default().with_app_reg_info(
    &AppRegInfoBuilder::default()
      .client_id(integ_test_config.resource_client_id.clone())
      .client_secret(integ_test_config.resource_client_secret.clone())
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
  Ok(state)
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

async fn create_token_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<TestTokenResponse>, String> {
  let app_service = state.app_service();
  let auth_service = app_service.auth_service();

  let app_reg_info = app_service
    .secret_service()
    .app_reg_info()
    .map_err(|e| e.to_string())?
    .ok_or("app_reg_info missing".to_string())?;

  let token = headers
    .get(KEY_RESOURCE_TOKEN)
    .ok_or("token missing".to_string())?
    .to_str()
    .map_err(|err| err.to_string())?;

  // Exchange token
  let (_, offline_token) = auth_service
    .exchange_token(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      token,
      "urn:ietf:params:oauth:token-type:refresh_token",
      vec![
        "openid".to_string(),
        "offline_access".to_string(),
        "scope_token_user".to_string(),
      ],
    )
    .await
    .map_err(|e| e.to_string())?;

  let offline_token = offline_token.ok_or("refresh token missing".to_string())?;
  Ok(Json(TestTokenResponse {
    token: Some(offline_token),
    scope: None,
  }))
}

fn create_token_create_router(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .merge(
      Router::new()
        .route("/create", post(create_token_handler))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware)),
    )
    .layer(state.app_service().session_service().session_layer())
    .with_state(state)
}

// OAuth token response structure
#[derive(Debug, serde::Deserialize)]
struct TokenResponse {
  access_token: String,
}

// Helper to get OAuth token using password grant (resource owner)
async fn get_user_token(
  client: &Client,
  config: &IntegTestConfig,
  client_id: &str,
  client_secret: &str,
  scopes: &[&str],
) -> Result<String, anyhow::Error> {
  let token_url = format!(
    "{}/realms/{}/protocol/openid-connect/token",
    config.auth_server_url, config.realm
  );
  let scope_string = if scopes.is_empty() {
    String::new()
  } else {
    scopes.join(" ")
  };
  let mut params = vec![
    ("grant_type", "password"),
    ("client_id", &client_id),
    ("client_secret", &client_secret),
    ("username", &config.username),
    ("password", &config.password),
  ];
  if !scope_string.is_empty() {
    params.push(("scope", &scope_string));
  }
  let response = client.post(&token_url).form(&params).send().await?;
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Token request failed: {}",
    response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );
  let token_response: TokenResponse = response.json().await?;
  Ok(token_response.access_token)
}

// --- Add IntegTestConfig fixture ---
#[derive(Debug, Clone)]
struct IntegTestConfig {
  auth_server_url: String,
  realm: String,
  app_client_id: String,
  app_client_secret: String,
  resource_client_id: String,
  resource_client_secret: String,
  username: String,
  password: String,
}

#[fixture]
#[once]
fn integ_test_config() -> IntegTestConfig {
  let env_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/.env.test"));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }
  IntegTestConfig {
    auth_server_url: std::env::var("INTEG_TEST_AUTH_SERVER_URL")
      .expect("INTEG_TEST_AUTH_SERVER_URL must be set"),
    realm: std::env::var("INTEG_TEST_AUTH_REALM").expect("INTEG_TEST_AUTH_REALM must be set"),
    app_client_id: std::env::var("INTEG_TEST_APP_CLIENT_ID")
      .expect("INTEG_TEST_APP_CLIENT_ID must be set"),
    app_client_secret: std::env::var("INTEG_TEST_APP_CLIENT_SECRET")
      .expect("INTEG_TEST_APP_CLIENT_SECRET must be set"),
    resource_client_id: std::env::var("INTEG_TEST_RESOURCE_CLIENT_ID")
      .expect("INTEG_TEST_RESOURCE_CLIENT_ID must be set"),
    resource_client_secret: std::env::var("INTEG_TEST_RESOURCE_CLIENT_SECRET")
      .expect("INTEG_TEST_RESOURCE_CLIENT_SECRET must be set"),
    username: std::env::var("INTEG_TEST_USER").expect("INTEG_TEST_USER must be set"),
    password: std::env::var("INTEG_TEST_PASSWORD").expect("INTEG_TEST_PASSWORD must be set"),
  }
}

#[rstest]
#[tokio::test]
#[awt]
#[anyhow_trace]
async fn test_offline_token_exchange_success(
  #[future] state: anyhow::Result<DefaultRouterState>,
  integ_test_config: &IntegTestConfig,
) -> anyhow::Result<()> {
  let client = Client::new();
  let user_token = get_user_token(
    &client,
    &integ_test_config,
    &integ_test_config.resource_client_id,
    &integ_test_config.resource_client_secret,
    &["openid", "email", "profile", "roles"],
  )
  .await?;
  let session_id = Id::default();
  let session_data = hashmap! {
    SESSION_KEY_ACCESS_TOKEN.to_string() => Value::String(user_token.clone()),
  };
  let mut record = Record {
    id: session_id,
    data: session_data,
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  let state = Arc::new(state?);
  state
    .app_service()
    .session_service()
    .get_session_store()
    .create(&mut record)
    .await?;
  let session_cookie = Cookie::build(("bodhiapp_session_id", session_id.to_string()))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Strict)
    .build();
  let router = create_token_create_router(state.clone());
  let request = Request::builder()
    .method("POST")
    .uri("/create")
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .body(axum::body::Body::empty())?;
  let response = router.oneshot(request).await?;
  assert_eq!(
    StatusCode::OK,
    response.status(),
    "Offline token create failed: {}",
    response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string())
  );
  let body: TestTokenResponse = response.json().await?;
  let claims = extract_claims::<OfflineClaims>(&body.token.unwrap())?;
  assert_eq!(claims.azp, integ_test_config.resource_client_id);
  let mut token_scopes = claims.scope.split_whitespace().collect::<Vec<&str>>();
  token_scopes.sort();
  assert_eq!(
    vec!["offline_access", "openid", "scope_token_user"],
    token_scopes,
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_cross_client_token_exchange_success(
  #[future] state: anyhow::Result<DefaultRouterState>,
  integ_test_config: &IntegTestConfig,
) -> anyhow::Result<()> {
  let client = Client::new();
  let user_token = get_user_token(
    &client,
    &integ_test_config,
    &integ_test_config.app_client_id,
    &integ_test_config.app_client_secret,
    &[
      "openid",
      "email",
      "profile",
      "roles",
      "scope_user_power_user",
    ],
  )
  .await?;
  let state = Arc::new(state?);
  let router = create_test_router(state.clone());
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(axum::body::Body::empty())?;
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
  assert_eq!(
    body.token.is_some(),
    true,
    "Expected X-Resource-Token header to be set"
  );
  assert_eq!(
    body.scope.is_some(),
    true,
    "Expected X-Resource-Scope header to be set"
  );
  // --- Decode JWT and assert claims ---
  let token = body.token.as_ref().unwrap();
  let claims = extract_claims::<Claims>(token)?;
  assert_eq!(
    claims.email, integ_test_config.username,
    "JWT email claim should match test user"
  );
  let mut scopes = claims.scope.split_whitespace().collect::<Vec<&str>>();
  scopes.sort();
  let mut expected_scope = vec![
    "email",
    "openid",
    "profile",
    "roles",
    "scope_user_power_user",
  ];
  expected_scope.sort();
  assert_eq!(scopes, expected_scope, "JWT scope should match expected");
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_cross_client_token_exchange_no_user_scope(
  #[future] state: anyhow::Result<DefaultRouterState>,
  integ_test_config: &IntegTestConfig,
) -> anyhow::Result<()> {
  let client = Client::new();
  let user_token = get_user_token(
    &client,
    &integ_test_config,
    &integ_test_config.app_client_id,
    &integ_test_config.app_client_secret,
    &[],
  )
  .await?;
  let state = Arc::new(state?);
  let router = create_test_router(state.clone());
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(axum::body::Body::empty())?;
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let err: OpenAIApiError = response.json().await?;
  assert_eq!(
    OpenAIApiError::new(
      ErrorBody::new(
        "user does not have any privileges on this system".to_string(),
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
#[awt]
#[tokio::test]
async fn test_cross_client_token_exchange_auth_service_error(
  #[future] state: anyhow::Result<DefaultRouterState>,
  integ_test_config: &IntegTestConfig,
) -> anyhow::Result<()> {
  let client = Client::new();
  let user_token = get_user_token(
    &client,
    &integ_test_config,
    &integ_test_config.app_client_id,
    &integ_test_config.app_client_secret,
    &[],
  )
  .await?;
  let state = Arc::new(state?);
  let router = create_test_router(state.clone());
  let request = Request::builder()
    .method("GET")
    .uri("/test")
    .header("Authorization", format!("Bearer {}", user_token))
    .body(axum::body::Body::empty())?;
  let response = router.oneshot(request).await?;
  assert!(
    response.status().is_client_error(),
    "Expected client error, got {}",
    response.status()
  );
  Ok(())
}
