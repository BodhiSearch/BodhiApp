use crate::{auth_callback_handler, auth_initiate_handler, generate_pkce, RedirectResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::optional_auth_middleware;
use axum::{
  http::{status::StatusCode, Request},
  middleware::from_fn_with_state,
  routing::post,
  Router,
};
use axum_test::TestServer;
use chrono::Utc;
use oauth2::{AccessToken, RefreshToken};
use objs::test_utils::temp_bodhi_home;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  test_utils::{
    build_token, AppServiceStub, AppServiceStubBuilder, SecretServiceStub, SessionTestExt,
    SettingServiceStub,
  },
  AppRegInfo, AppService, AuthServiceError, MockAuthService, SecretServiceExt,
  SqliteSessionService,
};
use services::{AppStatus, BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

#[rstest]
fn test_generate_pkce() {
  let (generated_verifier, challenge) = generate_pkce();
  assert_eq!(43, generated_verifier.len());
  assert_eq!(43, challenge.len());
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  // Create a token with the correct scope that matches what auth_initiate_handler uses
  let claims = json! {{
    "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": "test_issuer".to_string(),
    "sub": Uuid::new_v4().to_string(),
    "typ": "Bearer",
    "azp": "test_client_id",
    "session_state": Uuid::new_v4().to_string(),
    "scope": "email openid profile roles", // Sorted scope that matches auth_initiate_handler
    "sid": Uuid::new_v4().to_string(),
    "email_verified": true,
    "name": "Test User",
    "preferred_username": "testuser@email.com",
    "given_name": "Test",
    "family_name": "User",
    "email": "testuser@email.com"
  }};
  let (token, _) = build_token(claims)?;
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut mock_auth_service = MockAuthService::default();
  let token_clone = token.clone();
  mock_auth_service
    .expect_exchange_auth_code()
    .times(1)
    .return_once(move |_, _, _, _, _| {
      Ok((
        AccessToken::new(token_clone.clone()),
        RefreshToken::new("test_refresh_token".to_string()),
      ))
    });

  let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
    (BODHI_PORT.to_string(), "3000".to_string()),
  ]));

  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth_service))
    .setting_service(Arc::new(setting_service))
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()
    .await?;

  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
    .with_state(state)
    .layer(app_service.session_service().session_layer());

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  // Perform login request
  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Extract state and code_challenge from the login response
  let state = query_params.get("state").expect("state param missing");

  // Perform callback request
  let resp = client
    .post("/auth/callback")
    .json(&json! {{
      "code": "test_code",
      "state": state,
    }})
    .await;
  resp.assert_status(StatusCode::OK);
  let callback_body: RedirectResponse = resp.json();
  assert_eq!(
    "http://frontend.localhost:3000/ui/chat",
    callback_body.location
  );
  let session_id = resp.cookie("bodhiapp_session_id");
  let access_token = session_service
    .get_session_value(session_id.value(), "access_token")
    .await
    .expect("access_token not found in session");
  let access_token = access_token.as_str().expect("access_token not a string");
  assert_eq!(token, access_token);
  let refresh_token = session_service
    .get_session_value(session_id.value(), "refresh_token")
    .await
    .expect("refresh_token not found in session");
  let refresh_token = refresh_token.as_str().expect("refresh_token not a string");
  assert_eq!("test_refresh_token", refresh_token);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_state_not_in_session(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Ready);
  let secret_service = Arc::new(secret_service);
  let app_service: AppServiceStub = AppServiceStubBuilder::default()
    .secret_service(secret_service)
    .build_session_service(temp_bodhi_home.path().join("test.db"))
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);
  let resp = router
    .oneshot(Request::post("/auth/callback").json(json! {{
      "code": "test_code",
      "state": "test_state",
    }})?)
    .await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  let json = resp.json::<Value>().await?;
  assert_eq!(
    "login_error-session_info_not_found",
    json["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_with_loopback_callback_url(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // Create a token with the correct scope that matches what auth_initiate_handler uses
  let claims = json! {{
    "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": "test_issuer".to_string(),
    "sub": Uuid::new_v4().to_string(),
    "typ": "Bearer",
    "azp": "test_client_id",
    "session_state": Uuid::new_v4().to_string(),
    "scope": "email openid profile roles", // Sorted scope that matches auth_initiate_handler
    "sid": Uuid::new_v4().to_string(),
    "email_verified": true,
    "name": "Test User",
    "preferred_username": "testuser@email.com",
    "given_name": "Test",
    "family_name": "User",
    "email": "testuser@email.com"
  }};
  let (token, _) = build_token(claims)?;
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut mock_auth_service = MockAuthService::default();
  let token_clone = token.clone();
  mock_auth_service
    .expect_exchange_auth_code()
    .times(1)
    .return_once(move |_, _, _, _, _| {
      Ok((
        AccessToken::new(token_clone.clone()),
        RefreshToken::new("test_refresh_token".to_string()),
      ))
    });

  // Configure with 0.0.0.0 (loopback)
  let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
    (BODHI_PORT.to_string(), "1135".to_string()),
  ]));

  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth_service))
    .setting_service(Arc::new(setting_service))
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()
    .await?;

  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
    .with_state(state)
    .layer(app_service.session_service().session_layer());

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  // Perform login request with Host header
  let login_resp = client
    .post("/auth/initiate")
    .add_header("Host", "localhost:1135")
    .await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Verify callback URL uses localhost from Host header
  let callback_url = query_params
    .get("redirect_uri")
    .expect("redirect_uri param missing");
  assert_eq!("http://localhost:1135/ui/auth/callback", callback_url);

  // Extract state for the callback request
  let state = query_params.get("state").expect("state param missing");

  // Perform callback request
  let resp = client
    .post("/auth/callback")
    .json(&json! {{
      "code": "test_code",
      "state": state,
    }})
    .await;
  resp.assert_status(StatusCode::OK);
  let callback_body: RedirectResponse = resp.json();

  // Final redirect should use localhost from the callback URL
  assert_eq!("http://localhost:1135/ui/chat", callback_body.location);

  // Verify session contains access token
  let session_id = resp.cookie("bodhiapp_session_id");
  let access_token = session_service
    .get_session_value(session_id.value(), "access_token")
    .await
    .expect("access_token not found in session");
  let access_token = access_token.as_str().expect("access_token not a string");
  assert_eq!(token, access_token);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_state_mismatch(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service: AppServiceStub = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
    .with_state(state)
    .layer(app_service.session_service().session_layer());

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  let state = format!(
    "modified-{}",
    query_params.get("state").expect("state param missing")
  );

  let resp = client
    .post("/auth/callback")
    .json(&json! {{
      "code": "test_code",
      "state": state,
    }})
    .await;
  resp.assert_status(StatusCode::BAD_REQUEST);
  let error = resp.json::<Value>();
  assert_eq!(
    "login_error-state_digest_mismatch",
    error["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_auth_service_error(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let mut mock_auth_service = MockAuthService::new();
  mock_auth_service
    .expect_exchange_auth_code()
    .times(1)
    .return_once(|_, _, _, _, _| {
      Err(AuthServiceError::AuthServiceApiError {
        status: 500,
        body: "network error".to_string(),
      })
    });
  let app_service: AppServiceStub = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth_service))
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
    .with_state(state)
    .layer(app_service.session_service().session_layer());

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  // Simulate login to set up session
  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  let state = query_params
    .get("state")
    .expect("state param missing")
    .to_string();
  let code = "test_code".to_string();
  let resp = client
    .post("/auth/callback")
    .json(&json! {{
      "code": code,
      "state": state,
    }})
    .await;

  resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
  let error = resp.json::<Value>();
  assert_eq!(
    "auth_service_error-auth_service_api_error",
    error["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
