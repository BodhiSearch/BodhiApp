use super::{utils::generate_random_string, RouterStateFn};
use crate::utils::{decode_access_token, Claims};
use crate::{HttpError, HttpErrorBuilder};
use axum::{
  body::Body,
  extract::{Query, State},
  http::{
    header::{HeaderMap, LOCATION},
    StatusCode,
  },
  response::{IntoResponse, Response},
  Json,
};
use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::TokenData;
use oauth2::{
  url::ParseError, AccessToken, AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier,
  RedirectUrl,
};
use objs::AppRegInfo;
use serde::{Deserialize, Serialize};
use services::{
  get_secret, AuthServiceError, SecretServiceError, KEY_APP_REG_INFO, KEY_APP_STATUS,
  KEY_RESOURCE_TOKEN,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;

#[derive(Debug, Clone, thiserror::Error)]
pub enum LoginError {
  #[error("app registration info not found")]
  AppRegInfoNotFound,
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error("{0}")]
  SessionError(String),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
  #[error(transparent)]
  ParseError(#[from] ParseError),
}

impl From<tower_sessions::session::Error> for LoginError {
  fn from(err: tower_sessions::session::Error) -> Self {
    LoginError::SessionError(err.to_string())
  }
}

impl From<LoginError> for HttpError {
  fn from(err: LoginError) -> Self {
    match err {
      LoginError::AppRegInfoNotFound => HttpErrorBuilder::default()
        .internal_server(Some(&err.to_string()))
        .build()
        .unwrap(),
      LoginError::SecretServiceError(e) => e.into(),
      LoginError::SessionError(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err))
        .build()
        .unwrap(),
      LoginError::AuthServiceError(e) => e.into(),
      LoginError::ParseError(e) => HttpErrorBuilder::default()
        .internal_server(Some(&e.to_string()))
        .build()
        .unwrap(),
    }
  }
}

impl IntoResponse for LoginError {
  fn into_response(self) -> Response {
    let error_response = HttpErrorBuilder::default()
      .internal_server(Some(&self.to_string()))
      .build()
      .unwrap();
    error_response.into_response()
  }
}

pub async fn login_handler(
  session: Session,
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Response, LoginError> {
  let app_service = state.app_service();
  let env_service = app_service.env_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  if let Ok(Some(access_token)) = session.get::<String>("access_token").await {
    if auth_service
      .check_access_token(&AccessToken::new(access_token))
      .await?
    {
      let ui_home = format!("{}/ui/home", env_service.frontend_url());
      return Ok(
        Response::builder()
          .status(StatusCode::FOUND)
          .header("Location", ui_home)
          .body(Body::empty())
          .unwrap()
          .into_response(),
      );
    }
  }

  let app_ref_info = get_secret::<_, AppRegInfo>(secret_service, KEY_APP_REG_INFO)?
    .ok_or(LoginError::AppRegInfoNotFound)?;
  let callback_url = env_service.login_callback_url();
  let client_id = app_ref_info.client_id;
  let state = generate_random_string(32);
  session.insert("oauth_state", &state).await?;

  let (code_verifier, code_challenge) = generate_pkce();
  session.insert("pkce_verifier", &code_verifier).await?;

  let login_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope=openid+email+profile",
        env_service.login_url(), client_id, callback_url, state, code_challenge
    );

  let response = Response::builder()
    .status(StatusCode::FOUND)
    .header("Location", login_url)
    .body(Body::empty())
    .unwrap()
    .into_response();
  Ok(response)
}

pub async fn login_callback_handler(
  session: Session,
  State(state): State<Arc<dyn RouterStateFn>>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, LoginError> {
  let app_service = state.app_service();
  let env_service = app_service.env_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  let stored_state = session.get::<String>("oauth_state").await?.ok_or_else(|| {
    LoginError::SessionError("login info not found in session, are cookies enabled?".to_string())
  })?;
  let received_state = params
    .get("state")
    .ok_or_else(|| LoginError::SessionError("Missing state parameter".to_string()))?;
  if stored_state != *received_state {
    return Err(LoginError::SessionError(
      "state parameter in callback does not match with the one sent in login request".to_string(),
    ));
  }

  let code = params
    .get("code")
    .ok_or_else(|| LoginError::SessionError("Missing code parameter".to_string()))?;

  let pkce_verifier = session
    .get::<String>("pkce_verifier")
    .await?
    .ok_or_else(|| LoginError::SessionError("Missing pkce_verifier in session".to_string()))?;

  let app_reg_info = get_secret::<_, AppRegInfo>(secret_service.clone(), KEY_APP_REG_INFO)?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  let token_response = auth_service
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(app_reg_info.client_id),
      ClientSecret::new(app_reg_info.client_secret),
      RedirectUrl::new(env_service.login_callback_url())?,
      PkceCodeVerifier::new(pkce_verifier),
    )
    .await?;

  session.remove::<String>("oauth_state").await?;
  session.remove::<String>("pkce_verifier").await?;
  session
    .insert("access_token", token_response.0.secret())
    .await?;
  session
    .insert("refresh_token", token_response.1.secret())
    .await?;
  secret_service.set_secret_string(KEY_APP_STATUS, "ready")?;

  let ui_home = format!("{}/ui/home", env_service.frontend_url());
  Ok(
    Response::builder()
      .status(StatusCode::FOUND)
      .header("Location", ui_home)
      .body(Body::empty())
      .unwrap()
      .into_response(),
  )
}

fn generate_pkce() -> (String, String) {
  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  (code_verifier, code_challenge)
}

#[derive(Debug, thiserror::Error)]
pub enum LogoutError {
  #[error("failed to delete session: {0}")]
  SessionDeleteError(String),
}

impl From<LogoutError> for HttpError {
  fn from(value: LogoutError) -> Self {
    match value {
      LogoutError::SessionDeleteError(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err))
        .build()
        .unwrap(),
    }
  }
}

pub async fn logout_handler(
  session: Session,
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Response, HttpError> {
  let env_service = state.app_service().env_service();
  session
    .delete()
    .await
    .map_err(|err| LogoutError::SessionDeleteError(err.to_string()))?;
  let ui_login = format!("{}/ui/login", env_service.frontend_url());
  // TODO: sending 200 instead of 302 to avoid axios/xmlhttprequest following redirects
  let response = Response::builder()
    .status(StatusCode::OK)
    .header(LOCATION, ui_login)
    .body(Body::empty())
    .unwrap();
  Ok(response)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
  pub logged_in: bool,
  pub email: Option<String>,
}

pub async fn user_info_handler(headers: HeaderMap) -> Result<Json<UserInfo>, HttpError> {
  let not_loggedin = UserInfo {
    logged_in: false,
    email: None,
  };
  let Some(token) = headers.get(KEY_RESOURCE_TOKEN) else {
    return Ok(Json(not_loggedin));
  };
  let Ok(token_str) = token.to_str() else {
    return Ok(Json(not_loggedin));
  };
  if token_str.is_empty() {
    return Ok(Json(not_loggedin));
  }
  let token_data: TokenData<Claims> = decode_access_token(token_str).map_err(|err| {
    HttpErrorBuilder::default()
      .unauthorized(&format!("invalid token: {err}"), None)
      .build()
      .unwrap()
  })?;
  Ok(Json(UserInfo {
    logged_in: true,
    email: Some(token_data.claims.email),
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RouterState;
  use crate::test_utils::{MockSharedContext, ResponseTestExt};
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{header, Request},
    middleware::{from_fn, Next},
    routing::{get, post},
    Json, Router,
  };
  use axum_test::TestServer;
  use mockall::predicate::function;
  use oauth2::RefreshToken;
  use objs::test_utils::temp_bodhi_home;
  use rstest::rstest;
  use serde_json::{json, Value};
  use services::{
    test_utils::{
      token, AppServiceStub, AppServiceStubBuilder, EnvServiceStub, SecretServiceStub,
      SessionTestExt,
    },
    AppServiceFn, MockAuthService, MockEnvServiceFn, SqliteSessionService, BODHI_FRONTEND_URL,
  };
  use std::{collections::HashMap, sync::Arc};
  use strfmt::strfmt;
  use tempfile::TempDir;
  use tower::ServiceExt;
  use url::Url;

  #[rstest]
  #[case(
        SecretServiceStub::with_map(maplit::hashmap! {
            KEY_APP_REG_INFO.to_string() => serde_json::to_string(&AppRegInfo {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                public_key: "test_public_key".to_string(),
                alg: jsonwebtoken::Algorithm::RS256,
                kid: "test_kid".to_string(),
                issuer: "test_issuer".to_string(),
            }).unwrap(),
        }),
        "http://localhost:3000/callback",
        "http://test-id.getbodhi.app/realms/test-realm/auth",
    )]
  #[tokio::test]
  async fn test_login_handler(
    #[case] secret_service: SecretServiceStub,
    #[case] callback_url: &str,
    #[case] login_url: &str,
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env_service = MockEnvServiceFn::new();
    mock_env_service
      .expect_login_callback_url()
      .return_const(callback_url.to_string());
    mock_env_service
      .expect_login_url()
      .return_const(login_url.to_string());
    let dbfile = temp_bodhi_home.path().join("test.db");
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .env_service(Arc::new(mock_env_service))
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/login", get(login_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let resp = router
      .oneshot(Request::get("/login").body(Body::empty())?)
      .await?;

    let status = resp.status();
    let location = resp
      .headers()
      .get("location")
      .map(|h| h.to_str().unwrap())
      .unwrap_or("somevalue");
    assert!(location.starts_with(login_url));
    assert_eq!(status, StatusCode::FOUND);

    let url = Url::parse(location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query_params.get("response_type").unwrap(), "code");
    assert_eq!(query_params.get("client_id").unwrap(), "test_client_id");
    assert_eq!(query_params.get("redirect_uri").unwrap(), callback_url);
    assert!(query_params.contains_key("state"));
    assert!(query_params.contains_key("code_challenge"));
    assert_eq!(query_params.get("code_challenge_method").unwrap(), "S256");
    assert_eq!(query_params.get("scope").unwrap(), "openid email profile");

    Ok(())
  }

  async fn set_session_token(req: Request<Body>, next: Next) -> Response {
    let session = req
      .extensions()
      .get::<Session>()
      .ok_or(anyhow::anyhow!("Missing session"))
      .unwrap();
    session.insert("access_token", "valid_token").await.unwrap();
    next.run(req).await
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_handler_already_logged_in(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_check_access_token()
      .with(function(move |arg: &AccessToken| {
        arg.secret() == "valid_token"
      }))
      .return_once(|_| Ok(true));
    let dbfile = temp_bodhi_home.path().join("test.db");
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(
        EnvServiceStub::default().with_env(BODHI_FRONTEND_URL, "http://frontend.localhost:3000"),
      ))
      .auth_service(Arc::new(mock_auth_service))
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let session_layer = app_service.session_service().session_layer();
    let router = Router::new()
      .route("/login", get(login_handler))
      .layer(from_fn(set_session_token))
      .layer(session_layer)
      .with_state(state);
    let resp = router
      .oneshot(Request::get("/login").body(Body::empty())?)
      .await?;

    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(
      resp.headers().get("location").unwrap(),
      &"http://frontend.localhost:3000/ui/home"
    );

    Ok(())
  }

  #[rstest]
  fn test_generate_pkce() {
    let (generated_verifier, challenge) = generate_pkce();
    assert_eq!(generated_verifier.len(), 43);
    assert_eq!(challenge.len(), 43);
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_login_callback_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_auth_code()
      .returning(|_, _, _, _, _| {
        Ok((
          AccessToken::new("test_access_token".to_string()),
          RefreshToken::new("test_refresh_token".to_string()),
        ))
      });

    let mock_env_service =
      EnvServiceStub::default().with_env(BODHI_FRONTEND_URL, "http://frontend.localhost:3000");

    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_REG_INFO.to_string() => serde_json::to_string(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      }).unwrap(),
    });
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .env_service(Arc::new(mock_env_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .await
      .build()?;

    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/login", get(login_handler))
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.do_save_cookies();

    // Perform login request
    let login_resp = client.get("/login").await;
    login_resp.assert_status(StatusCode::FOUND);
    let location = login_resp.headers().get("location").unwrap().to_str()?;
    let url = Url::parse(location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Extract state and code_challenge from the login response
    let state = query_params.get("state").unwrap();
    let code_challenge = query_params.get("code_challenge").unwrap();

    let query = format!(
      "code=test_code&state={}&code_challenge={}",
      state, code_challenge
    );

    // Perform callback request
    let resp = client.get(&format!("/login/callback?{}", query)).await;
    resp.assert_status(StatusCode::FOUND);
    assert_eq!(
      resp.headers().get(header::LOCATION).unwrap(),
      "http://frontend.localhost:3000/ui/home"
    );
    let session_id = resp.cookie("bodhiapp_session_id");
    let access_token = session_service
      .get_session_value(session_id.value(), "access_token")
      .await
      .unwrap();
    let access_token = access_token.as_str().unwrap();
    assert_eq!(access_token, "test_access_token");
    let refresh_token = session_service
      .get_session_value(session_id.value(), "refresh_token")
      .await
      .unwrap();
    let refresh_token = refresh_token.as_str().unwrap();
    assert_eq!(refresh_token, "test_refresh_token");
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_login_callback_handler_state_not_in_session(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .build_session_service(temp_bodhi_home.path().join("test.db"))
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let resp = router
      .oneshot(Request::get("/login/callback?code=test_code&state=test_state").body(Body::empty())?)
      .await?;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = resp.json::<Value>().await?;
    assert_eq!(
      json["message"],
      "login info not found in session, are cookies enabled?"
    );
    Ok(())
  }

  #[rstest]
  #[case(
    "code=test_code&state={state}-modified&code_challenge={code_challenge}",
    "state parameter in callback does not match with the one sent in login request"
  )]
  #[case(
    "state={state}&code_challenge={code_challenge}",
    "Missing code parameter"
  )]
  #[tokio::test]
  async fn test_login_callback_handler_missing_params(
    temp_bodhi_home: TempDir,
    #[case] query_template: &str,
    #[case] expected_error: &str,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_REG_INFO.to_string() => serde_json::to_string(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      }).unwrap(),
    });
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/login", get(login_handler))
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.do_save_cookies();

    let login_resp = client.get("/login").await;
    login_resp.assert_status(StatusCode::FOUND);
    let location = login_resp.headers().get("location").unwrap().to_str()?;
    let url = Url::parse(location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let state = query_params.get("state").unwrap().to_string();
    let code_challenge = query_params.get("code_challenge").unwrap().to_string();
    let query = strfmt!(query_template, state, code_challenge)
      .unwrap()
      .to_string();
    let resp = client.get(&format!("/login/callback?{}", query)).await;
    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    let error = resp.json::<Value>();
    let error = error.get("message").unwrap().as_str().unwrap();
    assert_eq!(error, expected_error);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_callback_auth_service_error(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_REG_INFO.to_string() => serde_json::to_string(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      }).unwrap(),
    });
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let mut mock_auth_service = MockAuthService::new();
    mock_auth_service
      .expect_exchange_auth_code()
      .returning(|_, _, _, _, _| {
        Err(AuthServiceError::AuthServiceApiError(
          "network error".to_string(),
        ))
      });
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/login", get(login_handler))
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.do_save_cookies();

    // Simulate login to set up session
    let login_resp = client.get("/login").await;
    login_resp.assert_status(StatusCode::FOUND);
    let location = login_resp.headers().get("location").unwrap().to_str()?;
    let url = Url::parse(location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let state = query_params.get("state").unwrap().to_string();
    let code_challenge = query_params.get("code_challenge").unwrap().to_string();
    // Prepare callback request
    let callback_query = format!("code=test-code&state={state}&code_challenge={code_challenge}");
    let resp = client
      .get(&format!("/login/callback?{}", callback_query))
      .await;

    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    let error = resp.json::<Value>();
    let error = error.get("message").unwrap().as_str().unwrap();
    assert_eq!(error, "api_error: network error");
    Ok(())
  }

  pub async fn create_test_session_handler(session: Session) -> impl IntoResponse {
    session.insert("test", "test").await.unwrap();
    (StatusCode::CREATED, Json(json!({})))
  }

  #[rstest]
  #[tokio::test]
  async fn test_logout_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service =
      Arc::new(SqliteSessionService::build_session_service(dbfile.clone()).await);
    let app_service: Arc<dyn AppServiceFn> = Arc::new(
      AppServiceStubBuilder::default()
        .with_sqlite_session_service(session_service.clone())
        .await
        .build()?,
    );

    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/app/logout", post(logout_handler))
      .route("/test/session/new", post(create_test_session_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.do_save_cookies();

    let resp = client.post("/test/session/new").await;
    resp.assert_status(StatusCode::CREATED);
    let cookie = resp.cookie("bodhiapp_session_id");
    let session_id = cookie.value_trimmed();

    let record = session_service.get_session_record(session_id).await;
    assert!(record.is_some());

    let resp = client.post("/app/logout").await;
    resp.assert_status(StatusCode::OK);
    let location = resp.header("Location");
    let location = location.to_str().unwrap();
    assert_eq!("http://localhost:1135/ui/login", location);
    let record = session_service.get_session_record(session_id).await;
    assert!(record.is_none());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_valid_token(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token.unwrap();
    let app_service: Arc<dyn AppServiceFn> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/app/user", get(user_info_handler))
      .with_state(state);
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_RESOURCE_TOKEN, token)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = response.json::<Value>().await.unwrap();
    assert_eq!(
      "testuser@email.com",
      response_json["email"].as_str().unwrap(),
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_empty_token() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppServiceFn> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/app/user", get(user_info_handler))
      .with_state(state);
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_RESOURCE_TOKEN, "")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = response.json::<UserInfo>().await?;
    assert_eq!(
      UserInfo {
        logged_in: false,
        email: None,
      },
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_invalid_token() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppServiceFn> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/app/user", get(user_info_handler))
      .with_state(state);
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_RESOURCE_TOKEN, "invalid_token")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response_json = response.json::<Value>().await?;
    assert_eq!(
      "invalid token: InvalidToken",
      response_json["message"].as_str().unwrap()
    );
    Ok(())
  }
}
