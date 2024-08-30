use super::{utils::generate_random_string, RouterStateFn};
use crate::service::{
  get_secret, AppRegInfo, AuthServiceError, HttpError, HttpErrorBuilder, SecretServiceError,
  KEY_APP_REG_INFO,
};
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine as _};
use oauth2::AccessToken;
use sha2::{Digest, Sha256};
use std::sync::Arc;
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

fn generate_pkce() -> (String, String) {
  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  (code_verifier, code_challenge)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    server::RouterState,
    service::{AppServiceFn, MockAuthService, MockEnvServiceFn, BODHI_FRONTEND_URL},
    test_utils::{
      temp_bodhi_home, AppServiceStubBuilder, EnvServiceStub, MockSharedContext, SecretServiceStub,
    },
  };
  use axum::{
    body::Body,
    http::Request,
    middleware::{from_fn, Next},
    routing::get,
    Router,
  };
  use mockall::predicate::function;
  use rstest::rstest;
  use std::sync::Arc;
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
      .with_session_service(dbfile)
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
    let query_params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
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
      .with_session_service(dbfile)
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
}
