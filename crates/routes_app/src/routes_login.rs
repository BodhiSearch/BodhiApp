use crate::LoginError;
use auth_middleware::{app_status_or_default, generate_random_string};
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
  AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl, RefreshToken,
};
use objs::{ApiError, AppError, BadRequestError, ErrorType};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  decode_access_token, get_secret, AppRegInfo, AppStatus, Claims, KEY_APP_REG_INFO, KEY_APP_STATUS,
  KEY_RESOURCE_TOKEN,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;

// TODO: use `impl_error_from!`

pub async fn login_handler(
  headers: HeaderMap,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  let env_service = app_service.env_service();
  match headers.get(KEY_RESOURCE_TOKEN) {
    Some(_) => {
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
    None => {
      let secret_service = app_service.secret_service();
      let app_ref_info = get_secret::<_, AppRegInfo>(secret_service, KEY_APP_REG_INFO)?
        .ok_or(LoginError::AppRegInfoNotFound)?;
      let callback_url = env_service.login_callback_url();
      let client_id = app_ref_info.client_id;
      let state = generate_random_string(32);
      // TODO: use `impl_error_from!`
      session
        .insert("oauth_state", &state)
        .await
        .map_err(LoginError::from)?;

      let (code_verifier, code_challenge) = generate_pkce();
      // TODO: use `impl_error_from!`
      session
        .insert("pkce_verifier", &code_verifier)
        .await
        .map_err(LoginError::from)?;

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
  }
}

pub async fn login_callback_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  let env_service = app_service.env_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  let app_status = app_status_or_default(&secret_service);
  if app_status == AppStatus::Setup {
    return Err(LoginError::AppStatusInvalid(app_status))?;
  }
  let status_resource_admin = app_status == AppStatus::ResourceAdmin;
  let stored_state = session
    .get::<String>("oauth_state")
    .await
    .map_err(LoginError::from)?
    .ok_or_else(|| LoginError::SessionInfoNotFound)?;
  let received_state = params
    .get("state")
    .ok_or_else(|| BadRequestError::new("missing state parameter".to_string()))?;
  if stored_state != *received_state {
    return Err(
      BadRequestError::new(
        "state parameter in callback does not match with the one sent in login request".to_string(),
      )
      .into(),
    );
  }

  let code = params
    .get("code")
    .ok_or_else(|| BadRequestError::new("missing code parameter".to_string()))?;

  let pkce_verifier = session
    .get::<String>("pkce_verifier")
    .await
    .map_err(LoginError::from)?
    .ok_or(LoginError::SessionInfoNotFound)?;

  let app_reg_info = get_secret::<_, AppRegInfo>(&secret_service, KEY_APP_REG_INFO)?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  let token_response = auth_service
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(app_reg_info.client_id.clone()),
      ClientSecret::new(app_reg_info.client_secret.clone()),
      RedirectUrl::new(env_service.login_callback_url()).map_err(LoginError::from)?,
      PkceCodeVerifier::new(pkce_verifier),
    )
    .await?;

  session
    .remove::<String>("oauth_state")
    .await
    .map_err(LoginError::from)?;
  session
    .remove::<String>("pkce_verifier")
    .await
    .map_err(LoginError::from)?;
  let mut access_token = token_response.0.secret().to_string();
  let mut refresh_token = token_response.1.secret().to_string();
  let email = decode_access_token(&access_token)?.claims.email;
  if status_resource_admin {
    auth_service
      .make_resource_admin(&app_reg_info.client_id, &app_reg_info.client_secret, &email)
      .await?;
    secret_service.set_secret_string(KEY_APP_STATUS, &AppStatus::Ready.to_string())?;
    let (new_access_token, new_refresh_token) = auth_service
      .refresh_token(
        RefreshToken::new(refresh_token.to_string()),
        ClientId::new(app_reg_info.client_id.clone()),
        ClientSecret::new(app_reg_info.client_secret.clone()),
      )
      .await?;
    access_token = new_access_token.secret().to_string();
    refresh_token = new_refresh_token.secret().to_string();
  }
  session
    .insert("access_token", access_token)
    .await
    .map_err(LoginError::from)?;
  session
    .insert("refresh_token", refresh_token)
    .await
    .map_err(LoginError::from)?;
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

pub fn generate_pkce() -> (String, String) {
  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  (code_verifier, code_challenge)
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LogoutError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "logout_error-session_delete_error", args_delegate = false)]
  SessionDelete(#[from] tower_sessions::session::Error),
}

pub async fn logout_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let env_service = state.app_service().env_service();
  session.delete().await.map_err(LogoutError::from)?;
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

pub async fn user_info_handler(headers: HeaderMap) -> Result<Json<UserInfo>, ApiError> {
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
  let token_data: TokenData<Claims> = decode_access_token(token_str)?;
  Ok(Json(UserInfo {
    logged_in: true,
    email: Some(token_data.claims.email),
  }))
}

#[cfg(test)]
mod tests {
  use crate::{
    generate_pkce, login_callback_handler, login_handler, logout_handler, user_info_handler,
    UserInfo,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::optional_auth_middleware;
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
  };
  use axum_test::TestServer;
  use hyper::header::LOCATION;
  use mockito::{Matcher, Server};
  use oauth2::{AccessToken, PkceCodeVerifier, RefreshToken};
  use objs::{
    test_utils::{setup_l10n, temp_bodhi_home},
    FluentLocalizationService,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContextRw, RouterState,
  };
  use services::{
    test_utils::{
      expired_token, token, AppServiceStub, AppServiceStubBuilder, EnvServiceStub,
      SecretServiceStub, SessionTestExt,
    },
    AppRegInfo, AppService, AuthServiceError, MockAuthService, MockEnvService,
    SqliteSessionService, APP_STATUS_READY, BODHI_FRONTEND_URL, KEY_APP_REG_INFO, KEY_APP_STATUS,
    KEY_RESOURCE_TOKEN,
  };
  use services::{AppStatus, KeycloakAuthService};
  use std::{collections::HashMap, sync::Arc};
  use strfmt::strfmt;
  use tempfile::TempDir;
  use time::{Duration, OffsetDateTime};
  use tower::ServiceExt;
  use tower_sessions::{
    session::{Id, Record},
    Session, SessionStore,
  };
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
    let mut mock_env_service = MockEnvService::new();
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
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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

  #[rstest]
  #[tokio::test]
  async fn test_login_handler_logged_in_redirects_to_home(
    temp_bodhi_home: TempDir,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    test_login_handler_with_token(
      temp_bodhi_home,
      token,
      StatusCode::FOUND,
      "http://frontend.localhost:3000/ui/home",
    )
    .await
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_handler_for_expired_token_redirects_to_login(
    temp_bodhi_home: TempDir,
    expired_token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = expired_token;
    test_login_handler_with_token(
      temp_bodhi_home,
      token,
      StatusCode::FOUND,
      "http://id.localhost/realms/test-realm/protocol/openid-connect/auth",
    )
    .await
  }

  async fn test_login_handler_with_token(
    temp_bodhi_home: TempDir,
    token: String,
    status: StatusCode,
    location: &str,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = SqliteSessionService::build_session_service(dbfile).await;
    let record = set_token_in_session(&session_service, &token).await?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(
        EnvServiceStub::default().with_env(BODHI_FRONTEND_URL, "http://frontend.localhost:3000"),
      ))
      .with_sqlite_session_service(Arc::new(session_service))
      .with_secret_service()
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/login", get(login_handler))
      .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
      .with_state(state)
      .layer(app_service.session_service().session_layer());
    let resp = router
      .oneshot(
        Request::get("/login")
          .header("Cookie", format!("bodhiapp_session_id={}", record.id))
          .body(Body::empty())?,
      )
      .await?;
    assert_eq!(resp.status(), status);
    assert!(resp
      .headers()
      .get("location")
      .unwrap()
      .to_str()
      .unwrap()
      .starts_with(location),);
    Ok(())
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
    assert_eq!(generated_verifier.len(), 43);
    assert_eq!(challenge.len(), 43);
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_login_callback_handler(
    temp_bodhi_home: TempDir,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let mut mock_auth_service = MockAuthService::default();
    let token_clone = token.clone();
    mock_auth_service
      .expect_exchange_auth_code()
      .returning(move |_, _, _, _, _| {
        Ok((
          AccessToken::new(token_clone.clone()),
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
      KEY_APP_STATUS.to_string() => APP_STATUS_READY.to_string(),
    });
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .env_service(Arc::new(mock_env_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;

    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
      resp.headers().get(LOCATION).unwrap(),
      "http://frontend.localhost:3000/ui/home"
    );
    let session_id = resp.cookie("bodhiapp_session_id");
    let access_token = session_service
      .get_session_value(session_id.value(), "access_token")
      .await
      .unwrap();
    let access_token = access_token.as_str().unwrap();
    assert_eq!(access_token, token);
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
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let secret_service = Arc::new(SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => APP_STATUS_READY.to_string(),
    }));
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .secret_service(secret_service)
      .build_session_service(temp_bodhi_home.path().join("test.db"))
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
      json,
      json! {{
        "error": {
          "message": "login info not found in session, are cookies enabled?",
          "code": "login_error-session_info_not_found",
          "type": "internal_server_error"
        }
      }}
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
    "missing code parameter"
  )]
  #[tokio::test]
  async fn test_login_callback_handler_missing_params(
    temp_bodhi_home: TempDir,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
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
      KEY_APP_STATUS.to_string() => APP_STATUS_READY.to_string(),
    });
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
    resp.assert_status(StatusCode::BAD_REQUEST);
    let error = resp.json::<Value>();
    let expected_message =
      "invalid request, reason: \u{2068}".to_string() + expected_error + "\u{2069}";
    assert_eq!(
      error,
      json! {{
        "error": {
          "message": expected_message,
          "code": "bad_request_error",
          "type": "invalid_request_error"
        }}
      }
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_callback_auth_service_error(
    temp_bodhi_home: TempDir,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
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
      KEY_APP_STATUS.to_string() => APP_STATUS_READY.to_string(),
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
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
    assert_eq!(
      error,
      json! {{
        "error": {
          "message": "error from auth service: \u{2068}network error\u{2069}",
          "code": "auth_service_error-auth_service_api_error",
          "type": "internal_server_error"
        }
      }}
    );
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
    let app_service: Arc<dyn AppService> = Arc::new(
      AppServiceStubBuilder::default()
        .with_sqlite_session_service(session_service.clone())
        .build()?,
    );

    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
  async fn test_user_info_handler_invalid_token(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
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
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
      json! {{
        "error": {
          "message": "authentication token is invalid",
          "code": "json_web_token_error-InvalidToken",
          "type": "authentication_error"
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_callback_handler_resource_admin(
    temp_bodhi_home: TempDir,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let mut server = Server::new_async().await;
    let keycloak_url = server.url();
    let id = Id::default();
    let app_service =
      setup_app_service_resource_admin(&temp_bodhi_home, &id, &keycloak_url).await?;
    setup_keycloak_mocks_resource_admin(&mut server, &token).await;
    let result = execute_login_callback(&id, app_service.clone()).await?;
    assert_login_callback_result_resource_admin(result, app_service).await?;
    Ok(())
  }

  async fn setup_app_service_resource_admin(
    temp_bodhi_home: &TempDir,
    id: &Id,
    keycloak_url: &str,
  ) -> anyhow::Result<Arc<AppServiceStub>> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = SqliteSessionService::build_session_service(dbfile).await;
    let mut record = Record {
      id: id.clone(),
      data: maplit::hashmap! {
        "oauth_state".to_string() => Value::String("test_state".to_string()),
        "pkce_verifier".to_string() => Value::String("test_pkce_verifier".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    let session_service = Arc::new(session_service);
    let secret_service = Arc::new(SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => AppStatus::ResourceAdmin.to_string(),
      KEY_APP_REG_INFO.to_string() => serde_json::to_string(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })?,
    }));
    let auth_service = Arc::new(KeycloakAuthService::new(
      keycloak_url.to_string(),
      "test-realm".to_string(),
    ));
    let mock_env_service = Arc::new(
      EnvServiceStub::default()
        .with_env(BODHI_FRONTEND_URL, "http://frontend.localhost:3000")
        .with_env("BODHI_HOST", "localhost")
        .with_env("BODHI_PORT", "9000")
        .with_env("BODHI_AUTH_URL", keycloak_url),
    );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(secret_service)
      .auth_service(auth_service)
      .env_service(mock_env_service)
      .with_sqlite_session_service(session_service)
      .build()?;
    Ok(Arc::new(app_service))
  }

  async fn setup_keycloak_mocks_resource_admin(server: &mut Server, token: &str) {
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
          "http://localhost:9000/app/login/callback".into(),
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
        "/realms/test-realm/bodhi/clients/make-resource-admin",
      )
      .match_header("Authorization", "Bearer client_access_token")
      .match_body(Matcher::Json(json!({"username": "testuser@email.com"})))
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

  async fn execute_login_callback(
    id: &Id,
    app_service: Arc<AppServiceStub>,
  ) -> Result<Response, anyhow::Error> {
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::default()),
      app_service.clone(),
    ));
    let router: Router = Router::new()
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let request = Request::get("/login/callback?code=test_code&state=test_state")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(request).await?;
    Ok(response)
  }

  async fn assert_login_callback_result_resource_admin(
    response: Response,
    app_service: Arc<AppServiceStub>,
  ) -> anyhow::Result<()> {
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
      response.headers().get("Location").unwrap(),
      "http://frontend.localhost:3000/ui/home"
    );
    let secret_service = app_service.secret_service();
    let updated_status = secret_service
      .get_secret_string(KEY_APP_STATUS)
      .unwrap()
      .unwrap();
    assert_eq!("ready", updated_status);
    Ok(())
  }
}
