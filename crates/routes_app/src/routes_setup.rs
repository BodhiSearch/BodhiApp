use auth_middleware::app_status_or_default;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, ErrorType};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  set_secret, AppStatus, AuthServiceError, SecretServiceError, KEY_APP_AUTHZ, KEY_APP_REG_INFO,
  KEY_APP_STATUS,
};
use std::sync::Arc;

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
  let authz = secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or(Some("true".to_string()))
    .unwrap_or("true".to_string());
  let status = app_status_or_default(secret_service);
  let env_service = &state.app_service().env_service();
  Ok(Json(AppInfo {
    version: env_service.version(),
    status,
    authz: authz == "true",
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

pub async fn setup_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, ApiError>,
) -> Result<SetupResponse, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();
  let status = app_status_or_default(secret_service);
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
    let app_reg_info = auth_service.register_client(redirect_uris).await?;
    set_secret(secret_service, KEY_APP_REG_INFO, &app_reg_info)?;
    secret_service.set_secret_string(KEY_APP_AUTHZ, "true")?;
    secret_service.set_secret_string(KEY_APP_STATUS, "resource-admin")?;
    Ok(SetupResponse {
      status: AppStatus::ResourceAdmin,
    })
  } else {
    secret_service.set_secret_string(KEY_APP_AUTHZ, "false")?;
    secret_service.set_secret_string(KEY_APP_STATUS, "ready")?;
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
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContextRw};
  use services::{
    get_secret,
    test_utils::{AppServiceStubBuilder, SecretServiceStub},
    AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService, KEY_APP_AUTHZ,
    KEY_APP_REG_INFO, KEY_APP_STATUS,
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
    SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => "setup".to_string(),
      KEY_APP_AUTHZ.to_string() => "true".to_string(),
    }),
    AppInfo {
      version: "0.0.0".to_string(),
      status: AppStatus::Setup,
      authz: true,
    }
  )]
  #[case(
    SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => "setup".to_string(),
      KEY_APP_AUTHZ.to_string() => "false".to_string(),
    }),
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
      Arc::new(MockSharedContextRw::default()),
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
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "ready".to_string(),
      }),
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
      Arc::new(MockSharedContextRw::default()),
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
      .get_secret_string(KEY_APP_AUTHZ)
      .unwrap()
      .is_none());
    assert_eq!(
      secret_service.get_secret_string(KEY_APP_STATUS).unwrap(),
      Some("ready".to_string())
    );
    let app_reg_info = get_secret::<_, AppRegInfo>(secret_service, KEY_APP_REG_INFO)?;
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
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "setup".to_string(),
      }),
      SetupRequest { authz: false },
      AppStatus::Ready,
  )]
  #[case(
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "setup".to_string(),
      }),
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
      .returning(|_redirect_uris| {
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
      Arc::new(MockSharedContextRw::default()),
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
    assert_eq!(
      Some(expected_status.to_string()),
      secret_service.get_secret_string(KEY_APP_STATUS).unwrap(),
    );
    assert_eq!(
      secret_service.get_secret_string(KEY_APP_AUTHZ).unwrap(),
      Some(request.authz.to_string())
    );
    let app_reg_info = get_secret::<_, AppRegInfo>(secret_service, KEY_APP_REG_INFO)?;
    assert_eq!(request.authz, app_reg_info.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_register_resource_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
        KEY_APP_STATUS.to_string() => "setup".to_string(),
    });
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .once()
      .returning(|_redirect_uris| {
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
      Arc::new(MockSharedContextRw::default()),
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
      Arc::new(MockSharedContextRw::default()),
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
