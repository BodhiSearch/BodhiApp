use crate::{LoginError, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO};
use auth_middleware::{app_status_or_default, generate_random_string, KEY_RESOURCE_TOKEN};
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
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use objs::{ApiError, AppError, BadRequestError, ErrorType, OpenAIApiError};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{extract_claims, AppStatus, Claims, SecretServiceExt};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;
use utoipa::ToSchema;

pub async fn login_handler(
  headers: HeaderMap,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  let setting_service = app_service.setting_service();
  match headers.get(KEY_RESOURCE_TOKEN) {
    Some(_) => {
      let ui_home = format!("{}/ui/home", setting_service.frontend_url());
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
      let app_reg_info = secret_service
        .app_reg_info()?
        .ok_or(LoginError::AppRegInfoNotFound)?;
      let callback_url = setting_service.login_callback_url();
      let client_id = app_reg_info.client_id;
      let state = generate_random_string(32);
      session
        .insert("oauth_state", &state)
        .await
        .map_err(LoginError::from)?;

      let (code_verifier, code_challenge) = generate_pkce();
      session
        .insert("pkce_verifier", &code_verifier)
        .await
        .map_err(LoginError::from)?;

      let scope = ["openid", "email", "profile", "roles"].join("%20"); // manual url encoding for space
      let login_url = format!(
          "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope={}",
          setting_service.login_url(), client_id, callback_url, state, code_challenge, scope
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

// TODO: rather than returning ApiError, return to frontend with error message
pub async fn login_callback_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  let setting_service = app_service.setting_service();
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
    .ok_or(LoginError::SessionInfoNotFound)?;
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

  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  let token_response = auth_service
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(app_reg_info.client_id.clone()),
      ClientSecret::new(app_reg_info.client_secret.clone()),
      RedirectUrl::new(setting_service.login_callback_url()).map_err(LoginError::from)?,
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
  let email = extract_claims::<Claims>(&access_token)?.email;
  if status_resource_admin {
    auth_service
      .make_resource_admin(&app_reg_info.client_id, &app_reg_info.client_secret, &email)
      .await?;
    secret_service.set_app_status(&AppStatus::Ready)?;
    let (new_access_token, new_refresh_token) = auth_service
      .refresh_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        &refresh_token,
      )
      .await?;
    access_token = new_access_token;
    refresh_token =
      new_refresh_token.expect("refresh token is missing when refreshing an existing token");
  }
  session
    .insert("access_token", access_token)
    .await
    .map_err(LoginError::from)?;
  session
    .insert("refresh_token", refresh_token)
    .await
    .map_err(LoginError::from)?;
  let ui_setup_resume = format!("{}/ui/setup/download-models", setting_service.frontend_url());
  Ok(
    Response::builder()
      .status(StatusCode::FOUND)
      .header("Location", ui_setup_resume)
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
  #[error_meta(error_type = ErrorType::InternalServer, code = "logout_error-session_delete_error", args_delegate = false)]
  SessionDelete(#[from] tower_sessions::session::Error),
}

/// Logout the current user by destroying their session
#[utoipa::path(
    post,
    path = ENDPOINT_LOGOUT,
    tag = "auth",
    operation_id = "logoutUser",
    responses(
        (status = 200, description = "Logout successful, redirects to login page",
         headers(
             ("Location" = String, description = "Frontend login page URL")
         )
        ),
        (status = 500, description = "Session deletion failed", body = OpenAIApiError)
    )
)]
pub async fn logout_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let setting_service = state.app_service().setting_service();
  session.delete().await.map_err(LogoutError::from)?;
  let ui_login = format!("{}/ui/login", setting_service.frontend_url());
  // TODO: sending 200 instead of 302 to avoid axios/xmlhttprequest following redirects
  let response = Response::builder()
    .status(StatusCode::OK)
    .header(LOCATION, ui_login)
    .body(Body::empty())
    .unwrap();
  Ok(response)
}

/// Information about the currently logged in user
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({
    "logged_in": true,
    "email": "user@example.com",
    "roles": ["admin", "user"]
}))]
pub struct UserInfo {
  /// If user is logged in
  pub logged_in: bool,
  /// User's email address
  pub email: Option<String>,
  /// List of roles assigned to the user
  pub roles: Vec<String>,
}

/// Get information about the currently logged in user
#[utoipa::path(
    get,
    path = ENDPOINT_USER_INFO,
    tag = "auth",
    operation_id = "getCurrentUser",
    responses(
        (status = 200, description = "Returns current user information", body = UserInfo),
        (status = 500, description = "Error in extracting user info from token", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "token is invalid",
                 "type": "authentication_error",
                 "code": "token_error-invalid_token"
             }
         })
        )
    )
)]
pub async fn user_info_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<UserInfo>, ApiError> {
  let not_loggedin = UserInfo {
    logged_in: false,
    email: None,
    roles: Vec::new(),
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
  let claims: Claims = extract_claims::<Claims>(token_str)?;
  let roles = if let Ok(Some(reg_info)) = state.app_service().secret_service().app_reg_info() {
    claims
      .resource_access
      .get(&reg_info.client_id)
      .map(|resource| resource.roles.clone())
      .unwrap_or_default()
  } else {
    vec![]
  };

  Ok(Json(UserInfo {
    logged_in: true,
    email: Some(claims.email),
    roles,
  }))
}

#[cfg(test)]
mod tests {
  use crate::{
    generate_pkce, login_callback_handler, login_handler, logout_handler, user_info_handler,
    UserInfo,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::{inject_session_auth_info, KEY_RESOURCE_TOKEN};
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
  };
  use axum_test::TestServer;
  use chrono::Utc;
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
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::{
    test_utils::{
      app_reg_info, build_token, expired_token, test_auth_service, token, AppServiceStub,
      AppServiceStubBuilder, SecretServiceStub, SessionTestExt, SettingServiceStub, TEST_CLIENT_ID,
    },
    AppRegInfo, AppService, AuthServiceError, MockAuthService, SecretServiceExt,
    SqliteSessionService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  };
  use services::{AppStatus, BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
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
  use uuid::Uuid;

  #[rstest]
  #[case(
        SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                public_key: "test_public_key".to_string(),
                alg: jsonwebtoken::Algorithm::RS256,
                kid: "test_kid".to_string(),
                issuer: "test_issuer".to_string(),
            }),
    )]
  #[tokio::test]
  async fn test_login_handler(
    #[case] secret_service: SecretServiceStub,
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    use axum::routing::get;
    let callback_url = "http://localhost:3000/app/login/callback";
    let login_url = "http://test-id.getbodhi.app/realms/test-realm/protocol/openid-connect/auth";

    let setting_service = SettingServiceStub::new(HashMap::from([
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
    assert_eq!("code", query_params.get("response_type").unwrap());
    assert_eq!("test_client_id", query_params.get("client_id").unwrap());
    assert_eq!(callback_url, query_params.get("redirect_uri").unwrap());
    assert!(query_params.contains_key("state"));
    assert!(query_params.contains_key("code_challenge"));
    assert_eq!("S256", query_params.get("code_challenge_method").unwrap());
    assert_eq!(
      "openid email profile roles",
      query_params.get("scope").unwrap()
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_handler_logged_in_redirects_to_home(
    temp_bodhi_home: TempDir,
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
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
    expired_token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = expired_token;
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
      .setting_service(Arc::new(SettingServiceStub::default().with_settings(
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
      .route("/login", get(login_handler))
      .route_layer(from_fn_with_state(state.clone(), inject_session_auth_info))
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
    assert_eq!(43, generated_verifier.len());
    assert_eq!(43, challenge.len());
  }

  #[anyhow_trace]
  #[rstest]
  #[tokio::test]
  async fn test_login_callback_handler(
    temp_bodhi_home: TempDir,
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
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

    let setting_service = SettingServiceStub::default().with_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
    ]));

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
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
      "http://frontend.localhost:3000/ui/home",
      resp.headers().get(LOCATION).unwrap(),
    );
    let session_id = resp.cookie("bodhiapp_session_id");
    let access_token = session_service
      .get_session_value(session_id.value(), "access_token")
      .await
      .unwrap();
    let access_token = access_token.as_str().unwrap();
    assert_eq!(token, access_token);
    let refresh_token = session_service
      .get_session_value(session_id.value(), "refresh_token")
      .await
      .unwrap();
    let refresh_token = refresh_token.as_str().unwrap();
    assert_eq!("test_refresh_token", refresh_token);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_login_callback_handler_state_not_in_session(
    temp_bodhi_home: TempDir,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
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
      .route("/login/callback", get(login_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let resp = router
      .oneshot(Request::get("/login/callback?code=test_code&state=test_state").body(Body::empty())?)
      .await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
    let json = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "login info not found in session, are cookies enabled?",
          "code": "login_error-session_info_not_found",
          "type": "internal_server_error"
        }
      }},
      json
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
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
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
      json! {{
        "error": {
          "message": expected_message,
          "code": "bad_request_error",
          "type": "invalid_request_error"
        }
      }},
      error
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
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let mut mock_auth_service = MockAuthService::new();
    mock_auth_service
      .expect_exchange_auth_code()
      .times(1)
      .return_once(|_, _, _, _, _| {
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
    assert_eq!(
      json! {{
        "error": {
          "message": "error from auth service: \u{2068}network error\u{2069}",
          "code": "auth_service_error-auth_service_api_error",
          "type": "internal_server_error"
        }
      }},
      error
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
    token: (String, String),
    app_reg_info: AppRegInfo,
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(
          SecretServiceStub::default().with_app_reg_info(&app_reg_info),
        ))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
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
    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<Value>().await.unwrap();
    assert_eq!(
      json! {{
        "email": "testuser@email.com",
        "roles": ["resource_manager", "resource_power_user", "resource_user", "resource_admin"],
        "logged_in": true
      }},
      response_json,
    );
    Ok(())
  }

  #[rstest]
  #[case::resource_access_field_missing(json!{{}})]
  #[case::resource_access_field_null(json!{{"resource_access": {}}})]
  #[case::resource_access_some_other_resource_roles(json!{{"resource_access": {
        "some-other-test-resource": {
          "roles": ["resource_manager", "resource_power_user", "resource_user", "resource_admin"]
        }
      }}})]
  #[tokio::test]
  async fn test_user_info_handler_resource_access_invalid(
    #[case] claims: Value,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    app_reg_info: AppRegInfo,
  ) -> anyhow::Result<()> {
    let mut final_claims = json! {{
      "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": "https://id.mydomain.com/realms/myapp".to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": "Bearer",
      "azp": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid profile email",
      "sid": Uuid::new_v4().to_string(),
      "email_verified": true,
      "name": "Test User",
      "preferred_username": "testuser@email.com",
      "given_name": "Test",
      "family_name": "User",
      "email": "testuser@email.com",
    }};
    if !claims["resource_access"].is_null() {
      final_claims["resource_access"] = claims["resource_access"].clone();
    }
    let (token, _) = build_token(final_claims).unwrap();
    let app_service: Arc<dyn AppService> = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(
          SecretServiceStub::default().with_app_reg_info(&app_reg_info),
        ))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
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
    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<Value>().await.unwrap();
    assert_eq!(
      json! {{
        "email": "testuser@email.com",
        "roles": [],
        "logged_in": true
      }},
      response_json,
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_empty_token() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(DefaultRouterState::new(
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
    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserInfo>().await?;
    assert_eq!(
      UserInfo {
        logged_in: false,
        email: None,
        roles: Vec::new(),
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
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "token is invalid: \u{2068}malformed token format\u{2069}",
          "code": "token_error-invalid_token",
          "type": "authentication_error"
        }
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_login_callback_handler_resource_admin(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    temp_bodhi_home: TempDir,
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
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
      id: *id,
      data: maplit::hashmap! {
        "oauth_state".to_string() => Value::String("test_state".to_string()),
        "pkce_verifier".to_string() => Value::String("test_pkce_verifier".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    let session_service = Arc::new(session_service);
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })
      .with_app_status(&AppStatus::ResourceAdmin);
    let secret_service = Arc::new(secret_service);
    let auth_service = Arc::new(test_auth_service(&keycloak_url));
    let setting_service = Arc::new(SettingServiceStub::default().with_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
      (BODHI_AUTH_URL.to_string(), keycloak_url.to_string()),
    ])));
    let app_service = AppServiceStubBuilder::default()
      .secret_service(secret_service)
      .auth_service(auth_service)
      .setting_service(setting_service)
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
          "http://frontend.localhost:3000/app/login/callback".into(),
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
      Arc::new(MockSharedContext::default()),
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
    assert_eq!(StatusCode::FOUND, response.status());
    assert_eq!(
      "http://frontend.localhost:3000/ui/home",
      response.headers().get("Location").unwrap(),
    );
    let secret_service = app_service.secret_service();
    let updated_status = secret_service.app_status().unwrap();
    assert_eq!(AppStatus::Ready, updated_status);
    Ok(())
  }
}
