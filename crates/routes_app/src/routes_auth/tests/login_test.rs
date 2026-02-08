use crate::{
  auth_callback_handler, auth_initiate_handler, generate_pkce, logout_handler, RedirectResponse,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{generate_random_string, inject_optional_auth_info};
use axum::body::to_bytes;
use axum::{
  http::{status::StatusCode, Request},
  middleware::from_fn_with_state,
  response::{IntoResponse, Response},
  routing::post,
  Json, Router,
};
use axum_test::TestServer;
use chrono::Utc;
use mockito::{Matcher, Server};
use oauth2::{AccessToken, PkceCodeVerifier, RefreshToken};
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
    build_token, expired_token, test_auth_service, token, AppServiceStub, AppServiceStubBuilder,
    SecretServiceStub, SessionTestExt, SettingServiceStub,
  },
  AppRegInfo, AppService, AuthServiceError, MockAuthService, SecretServiceExt,
  SqliteSessionService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use services::{AppStatus, BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  Session, SessionStore,
};
use url::Url;
use uuid::Uuid;

#[rstest]
#[case(
      SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
              client_id: "test_client_id".to_string(),
              client_secret: "test_client_secret".to_string(),
          }),
  )]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler(
  #[case] secret_service: SecretServiceStub,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let callback_url = "http://localhost:3000/ui/auth/callback";
  let login_url = "http://test-id.getbodhi.app/realms/test-realm/protocol/openid-connect/auth";

  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "localhost".to_string()),
    (BODHI_PORT.to_string(), "3000".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));
  let dbfile = temp_bodhi_home.path().join("test.db");
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/auth/initiate").json(json! {{}})?)
    .await?;

  let status = resp.status();
  assert_eq!(status, StatusCode::CREATED);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  assert!(body.location.starts_with(login_url));

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  assert_eq!(Some("code"), query_params.get("response_type").map(|s| s.as_str()));
  assert_eq!(Some("test_client_id"), query_params.get("client_id").map(|s| s.as_str()));
  assert_eq!(Some(callback_url), query_params.get("redirect_uri").map(|s| s.as_str()));
  assert!(query_params.contains_key("state"));
  assert!(query_params.contains_key("code_challenge"));
  assert_eq!(Some("S256"), query_params.get("code_challenge_method").map(|s| s.as_str()));
  assert_eq!(
    Some("openid email profile roles"),
    query_params.get("scope").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_loopback_host_detection(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let secret_service = SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
    client_id: "test_client_id".to_string(),
    client_secret: "test_client_secret".to_string(),
  });

  // Configure with default 0.0.0.0 host (loopback)
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
    (BODHI_PORT.to_string(), "1135".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));

  let dbfile = temp_bodhi_home.path().join("test.db");
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  // Request with localhost:1135 Host header
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Host", "localhost:1135")
        .json(json! {{}})?,
    )
    .await?;

  assert_eq!(StatusCode::CREATED, resp.status());
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Should use localhost from Host header instead of configured 0.0.0.0
  assert_eq!(
    Some("http://localhost:1135/ui/auth/callback"),
    query_params.get("redirect_uri").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_network_host_usage(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let secret_service = SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
    client_id: "test_client_id".to_string(),
    client_secret: "test_client_secret".to_string(),
  });

  // Configure with default settings (no explicit public host)
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
    (BODHI_PORT.to_string(), "1135".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));

  let dbfile = temp_bodhi_home.path().join("test.db");
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  // Request with network host header
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Host", "192.168.1.100:1135")
        .json(json! {{}})?,
    )
    .await?;

  assert_eq!(StatusCode::CREATED, resp.status());
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Should now use the request host for network installation support
  assert_eq!(
    Some("http://192.168.1.100:1135/ui/auth/callback"),
    query_params.get("redirect_uri").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_logged_in_redirects_to_home(
  temp_bodhi_home: TempDir,
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
  assert_eq!(status, StatusCode::OK);
  assert!(
    body
      .location
      .starts_with("http://frontend.localhost:3000/ui/chat"),
    "{} does not start with http://frontend.localhost:3000/ui/chat",
    body.location
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_for_expired_token_redirects_to_login(
  temp_bodhi_home: TempDir,
  expired_token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = expired_token;
  let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
  assert_eq!(status, StatusCode::CREATED);
  assert!(body
    .location
    .starts_with("http://id.localhost/realms/test-realm/protocol/openid-connect/auth"));
  Ok(())
}

async fn auth_initiate_handler_with_token_response(
  temp_bodhi_home: TempDir,
  token: String,
) -> anyhow::Result<(StatusCode, RedirectResponse)> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = SqliteSessionService::build_session_service(dbfile).await;
  let record = set_token_in_session(&session_service, &token).await?;
  let app_service = AppServiceStubBuilder::default()
    .with_temp_home_as(temp_bodhi_home)
    .setting_service(Arc::new(SettingServiceStub::default().append_settings(
      HashMap::from([
        (BODHI_SCHEME.to_string(), "http".to_string()),
        (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
        (BODHI_PORT.to_string(), "3000".to_string()),
      ]),
    )))
    .with_sqlite_session_service(Arc::new(session_service))
    .with_secret_service()
    .with_db_service()
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route_layer(from_fn_with_state(state.clone(), inject_optional_auth_info))
    .with_state(state)
    .layer(app_service.session_service().session_layer());
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Cookie", format!("bodhiapp_session_id={}", record.id))
        .header("Sec-Fetch-Site", "same-origin")
        .json(json! {{}})?,
    )
    .await?;
  let status = resp.status();
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  Ok((status, body))
}

async fn set_token_in_session(
  session_service: &SqliteSessionService,
  token: &str,
) -> Result<Record, anyhow::Error> {
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + time::Duration::days(1),
  };
  session_service.session_store.create(&mut record).await?;
  Ok(record)
}

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
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth_service))
    .setting_service(Arc::new(setting_service))
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()?;

  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

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
    .build()?;
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
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth_service))
    .setting_service(Arc::new(setting_service))
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()?;

  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

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
  let callback_url = query_params.get("redirect_uri").expect("redirect_uri param missing");
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
    })
    .with_app_status(&AppStatus::Ready);
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let app_service: AppServiceStub = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .with_sqlite_session_service(session_service.clone())
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  let state = format!("modified-{}", query_params.get("state").expect("state param missing"));

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
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  // Simulate login to set up session
  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);
  let body: RedirectResponse = login_resp.json();
  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  let state = query_params.get("state").expect("state param missing").to_string();
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

pub async fn create_test_session_handler(session: Session) -> impl IntoResponse {
  session.insert("test", "test").await.unwrap();
  (StatusCode::CREATED, Json(json!({})))
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_logout_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile.clone()).await);
  let app_service: Arc<dyn AppService> = Arc::new(
    AppServiceStubBuilder::default()
      .with_sqlite_session_service(session_service.clone())
      .build()?,
  );

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/app/logout", post(logout_handler))
    .route("/test/session/new", post(create_test_session_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  let resp = client.post("/test/session/new").await;
  resp.assert_status(StatusCode::CREATED);
  let cookie = resp.cookie("bodhiapp_session_id");
  let session_id = cookie.value_trimmed();

  let record = session_service.get_session_record(session_id).await;
  assert!(record.is_some());

  let resp = client.post("/app/logout").await;
  resp.assert_status(StatusCode::OK);
  let body: RedirectResponse = resp.json();
  assert_eq!("http://localhost:1135/ui/login", body.location);
  let record = session_service.get_session_record(session_id).await;
  assert!(record.is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_resource_admin(
  temp_bodhi_home: TempDir,
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let mut server = Server::new_async().await;
  let auth_server_url = server.url();
  let id = Id::default();
  let state = generate_random_string(32); // Use simple random state like the actual implementation
  let app_service =
    setup_app_service_resource_admin(&temp_bodhi_home, &id, &auth_server_url, &state).await?;
  setup_auth_server_mocks_resource_admin(&mut server, &token).await;
  let result = execute_auth_callback(&id, app_service.clone(), &state).await?;
  assert_login_callback_result_resource_admin(result, app_service).await?;
  Ok(())
}

async fn setup_app_service_resource_admin(
  temp_bodhi_home: &TempDir,
  id: &Id,
  auth_server_url: &str,
  state: &str,
) -> anyhow::Result<Arc<AppServiceStub>> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = SqliteSessionService::build_session_service(dbfile).await;
  let mut record = Record {
    id: *id,
    data: maplit::hashmap! {
      "oauth_state".to_string() => Value::String(state.to_string()),
      "pkce_verifier".to_string() => Value::String("test_pkce_verifier".to_string()),
      "callback_url".to_string() => Value::String(format!("http://frontend.localhost:3000/ui/auth/callback")),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service.session_store.create(&mut record).await?;
  let session_service = Arc::new(session_service);
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    })
    .with_app_status(&AppStatus::ResourceAdmin);
  let secret_service = Arc::new(secret_service);
  let auth_service = Arc::new(test_auth_service(auth_server_url));
  let setting_service = Arc::new(
    SettingServiceStub::default().append_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
      (BODHI_AUTH_URL.to_string(), auth_server_url.to_string()),
    ])),
  );
  let app_service = AppServiceStubBuilder::default()
    .secret_service(secret_service)
    .auth_service(auth_service)
    .setting_service(setting_service)
    .with_sqlite_session_service(session_service)
    .build()?;
  Ok(Arc::new(app_service))
}

async fn setup_auth_server_mocks_resource_admin(server: &mut Server, token: &str) {
  // Mock token endpoint for code exchange
  let code_verifier = PkceCodeVerifier::new("test_pkce_verifier".to_string());
  let code_secret = code_verifier.secret();
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
      Matcher::UrlEncoded("code".into(), "test_code".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      Matcher::UrlEncoded(
        "redirect_uri".into(),
        "http://frontend.localhost:3000/ui/auth/callback".into(),
      ),
      Matcher::UrlEncoded("code_verifier".into(), code_secret.into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": token,
        "refresh_token": "initial_refresh_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;

  // Mock token endpoint for client credentials
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": "client_access_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;

  // Mock make-resource-admin endpoint
  server
    .mock(
      "POST",
      "/realms/test-realm/bodhi/resources/make-resource-admin",
    )
    .match_header("Authorization", "Bearer client_access_token")
    .match_body(Matcher::Regex(r#"\{"user_id":"[^"]+"\}"#.to_string()))
    .with_status(200)
    .with_body("{}")
    .create_async()
    .await;

  // Mock token refresh endpoint
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
      Matcher::UrlEncoded("refresh_token".into(), "initial_refresh_token".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": "new_access_token",
        "refresh_token": "new_refresh_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;
}

async fn execute_auth_callback(
  id: &Id,
  app_service: Arc<AppServiceStub>,
  request_state: &str,
) -> Result<Response, anyhow::Error> {
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router: Router = Router::new()
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);
  let request = Request::post("/auth/callback")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .json(json! {{
      "code": "test_code",
      "state": request_state,
    }})?;
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(response)
}

async fn assert_login_callback_result_resource_admin(
  response: Response,
  app_service: Arc<AppServiceStub>,
) -> anyhow::Result<()> {
  let body_bytes = to_bytes(response.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  assert_eq!(
    "http://frontend.localhost:3000/ui/setup/download-models",
    body.location
  );
  let secret_service = app_service.secret_service();
  let updated_status = secret_service.app_status()?;
  assert_eq!(AppStatus::Ready, updated_status);
  Ok(())
}
