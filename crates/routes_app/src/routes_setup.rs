use crate::{generate_pkce, LoginError};
use auth_middleware::generate_random_string;
use axum::{
  body::Body,
  extract::{Query, State},
  http::{header::HeaderMap, StatusCode},
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use oauth2::{AuthorizationCode, ClientId, PkceCodeVerifier, RedirectUrl};
use objs::{ApiError, AppError, BadRequestError, ErrorType};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  AppStatus, AuthServiceError, SecretServiceError, SecretServiceExt, KEY_RESOURCE_TOKEN,
};
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppServiceError {
  #[error("already_setup")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  AlreadySetup,
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AppInfo {
  version: String,
  status: AppStatus,
  authz: bool,
}

pub async fn app_info_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<AppInfo>, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let authz = secret_service.authz_or_default();
  let status = secret_service.app_status().unwrap_or_default();
  let env_service = &state.app_service().env_service();
  Ok(Json(AppInfo {
    version: env_service.version(),
    status,
    authz,
  }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupRequest {
  authz: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupResponse {
  status: AppStatus,
}

impl IntoResponse for SetupResponse {
  fn into_response(self) -> Response {
    (StatusCode::OK, Json(self)).into_response()
  }
}

pub async fn setup_login_redirect(
  headers: HeaderMap,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let env_service = &state.app_service().env_service();
  let status = secret_service.app_status().unwrap_or_default();
  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }
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
      let callback_url = env_service.setup_callback_url();
      let client_id = env_service.client_id_bodhi_account();
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
      let login_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope=openid+email+profile",
        env_service.login_url(), client_id, callback_url, state, code_challenge
    );

      Ok(
        Response::builder()
          .status(StatusCode::FOUND)
          .header("Location", login_url)
          .body(Body::empty())
          .unwrap()
          .into_response(),
      )
    }
  }
}

pub async fn setup_callback_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  let env_service = app_service.env_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  let app_status = secret_service.app_status().unwrap_or_default();
  if app_status != AppStatus::Setup {
    return Err(LoginError::AppStatusInvalid(app_status))?;
  }
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

  let client_id = env_service.client_id_bodhi_account();
  let (access_token, _) = auth_service
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(client_id),
      None,
      RedirectUrl::new(env_service.setup_callback_url()).map_err(LoginError::from)?,
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

  let server_host = env_service.host();
  let is_loopback =
    server_host == "localhost" || server_host == "127.0.0.1" || server_host == "0.0.0.0";
  let hosts = if is_loopback {
    vec!["localhost", "127.0.0.1", "0.0.0.0"]
  } else {
    vec![server_host.as_str()]
  };
  let scheme = env_service.scheme();
  let port = env_service.port();
  let redirect_uris = hosts
    .into_iter()
    .map(|host| format!("{scheme}://{host}:{port}/app/login/callback"))
    .collect::<Vec<String>>();

  let app_reg_info = auth_service
    .register_client(access_token.secret().as_str(), redirect_uris)
    .await?;

  let (access_token, refresh_token) = auth_service
    .exchange_for_resource_token(
      access_token.secret().as_str(),
      env_service.client_id_bodhi_account().as_str(),
      app_reg_info.client_id.as_str(),
      app_reg_info.client_secret.as_str(),
    )
    .await?;
  session
    .insert("access_token", access_token.secret().as_str())
    .await
    .map_err(LoginError::from)?;
  session
    .insert("refresh_token", refresh_token.secret().as_str())
    .await
    .map_err(LoginError::from)?;
  secret_service.set_app_status(&AppStatus::Ready)?;
  secret_service.set_authz(true)?;
  secret_service.set_app_reg_info(&app_reg_info)?;
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

pub async fn setup_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, ApiError>,
) -> Result<SetupResponse, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();
  let status = secret_service.app_status().unwrap_or_default();
  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }
  if request.authz {
    let env_service = &state.app_service().env_service();
    let server_host = env_service.host();
    let is_loopback =
      server_host == "localhost" || server_host == "127.0.0.1" || server_host == "0.0.0.0";
    let hosts = if is_loopback {
      vec!["localhost", "127.0.0.1", "0.0.0.0"]
    } else {
      vec![server_host.as_str()]
    };
    let scheme = env_service.scheme();
    let port = env_service.port();
    let redirect_uris = hosts
      .into_iter()
      .map(|host| format!("{scheme}://{host}:{port}/app/login/callback"))
      .collect::<Vec<String>>();
    let app_reg_info = auth_service.register_client("", redirect_uris).await?;
    secret_service.set_app_reg_info(&app_reg_info)?;
    secret_service.set_authz(true)?;
    secret_service.set_app_status(&AppStatus::ResourceAdmin)?;
    Ok(SetupResponse {
      status: AppStatus::ResourceAdmin,
    })
  } else {
    secret_service.set_authz(false)?;
    secret_service.set_app_status(&AppStatus::Ready)?;
    Ok(SetupResponse {
      status: AppStatus::Ready,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{app_info_handler, setup_handler, AppInfo, SetupRequest};
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
  };
  use jsonwebtoken::Algorithm;
  use objs::{test_utils::setup_l10n, FluentLocalizationService, ReqwestError};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    test_utils::{AppServiceStubBuilder, SecretServiceStub},
    AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService, SecretServiceExt,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  #[rstest]
  #[case(
    SecretServiceStub::new(),
    AppInfo {
      version: "0.0.0".to_string(),
      status: AppStatus::Setup,
      authz: true,
    }
  )]
  #[case(
    SecretServiceStub::new().with_app_status_setup().with_app_authz_enabled(),
    AppInfo {
      version: "0.0.0".to_string(),
      status: AppStatus::Setup,
      authz: true,
    }
  )]
  #[case(
    SecretServiceStub::new().with_app_status_ready().with_app_authz_disabled(),
    AppInfo {
      version: "0.0.0".to_string(),
      status: AppStatus::Setup,
      authz: false,
    }
  )]
  #[tokio::test]
  async fn test_app_info_handler(
    #[case] secret_service: SecretServiceStub,
    #[case] expected: AppInfo,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service),
    ));
    let router = Router::new()
      .route("/app/info", get(app_info_handler))
      .with_state(state);
    let resp = router
      .oneshot(Request::get("/app/info").body(Body::empty())?)
      .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let value = resp.json::<AppInfo>().await?;
    assert_eq!(expected, value);
    Ok(())
  }

  #[rstest]
  #[case(
      SecretServiceStub::new().with_app_status_ready(),
      SetupRequest { authz: true },
  )]
  #[tokio::test]
  async fn test_setup_handler_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[case] secret_service: SecretServiceStub,
    #[case] payload: SetupRequest,
  ) -> anyhow::Result<()> {
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(MockAuthService::new()))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let resp = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))?,
      )
      .await?;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = resp.json::<Value>().await?;
    assert_eq!(
      body,
      json! {{
        "error": {
          "message": "app is already setup",
          "code": "app_service_error-already_setup",
          "type": "invalid_request_error",
        }
      }}
    );

    let secret_service = app_service.secret_service();
    assert!(secret_service
      .get_secret_string("KEY_APP_AUTHZ")
      .unwrap()
      .is_none());
    assert_eq!(secret_service.app_status().unwrap(), AppStatus::Ready);
    let app_reg_info = secret_service.app_reg_info()?;
    assert!(app_reg_info.is_none());
    Ok(())
  }

  #[rstest]
  #[case(
      SecretServiceStub::new(),
      SetupRequest { authz: false },
      AppStatus::Ready,
  )]
  #[case(
      SecretServiceStub::new(),
      SetupRequest { authz: true },
      AppStatus::ResourceAdmin,
  )]
  #[case(
      SecretServiceStub::new().with_app_status_setup(),
      SetupRequest { authz: false },
      AppStatus::Ready,
  )]
  #[case(
    SecretServiceStub::new().with_app_status_setup(),
      SetupRequest { authz: true },
      AppStatus::ResourceAdmin,
  )]
  #[tokio::test]
  async fn test_setup_handler_success(
    #[case] secret_service: SecretServiceStub,
    #[case] request: SetupRequest,
    #[case] expected_status: AppStatus,
  ) -> anyhow::Result<()> {
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .returning(|_token, _redirect_uris| {
        Ok(AppRegInfo {
          public_key: "public_key".to_string(),
          alg: Algorithm::RS256,
          kid: "kid".to_string(),
          issuer: "issuer".to_string(),
          client_id: "client_id".to_string(),
          client_secret: "client_secret".to_string(),
        })
      });
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let response = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&request)?))?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let secret_service = app_service.secret_service();
    assert_eq!(expected_status, secret_service.app_status().unwrap(),);
    assert_eq!(secret_service.authz().unwrap(), request.authz);
    let app_reg_info = secret_service.app_reg_info()?;
    assert_eq!(request.authz, app_reg_info.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_register_resource_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status_setup();
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .once()
      .returning(|_token, _redirect_uris| {
        Err(AuthServiceError::Reqwest(ReqwestError::new(
          "failed to register as resource server".to_string(),
        )))
      });
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let resp = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&SetupRequest {
            authz: true,
          })?))?,
      )
      .await?;

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = resp.json::<Value>().await?;
    assert_eq!(
      body,
      json! {{
        "error": {
          "message": "error connecting to internal service: \u{2068}failed to register as resource server\u{2069}",
          "code": "reqwest_error",
          "type": "internal_server_error",
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[case(
    r#"{"authz": "invalid"}"#,
    "failed to parse the request body as JSON, error: \u{2068}Failed to deserialize the JSON body into the target type\u{2069}"
  )]
  #[case(
    r#"{}"#,
    "failed to parse the request body as JSON, error: \u{2068}Failed to deserialize the JSON body into the target type\u{2069}"
  )]
  #[case(
    r#"{"authz": true,}"#,
    "failed to parse the request body as JSON, error: \u{2068}Failed to parse the request body as JSON\u{2069}"
  )]
  #[tokio::test]
  async fn test_setup_handler_bad_request(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[case] body: &str,
    #[case] expected_error: &str,
  ) -> anyhow::Result<()> {
    let app_service = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let resp = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .body(Body::from(body.to_string()))?,
      )
      .await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = resp.json::<Value>().await?;
    assert_eq!(
      body,
      json! {{
        "error": {
          "message": expected_error,
          "type": "invalid_request_error",
          "code": "json_rejection_error"
        }
      }}
    );
    Ok(())
  }
}
