use crate::{ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_PING};
use auth_middleware::app_status_or_default;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, ErrorType, OpenAIApiError};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{AppStatus, AuthServiceError, SecretServiceError, SecretServiceExt};
use std::sync::Arc;

use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppServiceError {
  #[error("already_setup")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AlreadySetup,
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}

/// Application information and status
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({
    "version": "0.1.0",
    "status": "ready"
}))]
pub struct AppInfo {
  /// Application version
  pub version: String,
  /// Current application status
  pub status: AppStatus,
}

#[utoipa::path(
    get,
    path = ENDPOINT_APP_INFO,
    tag = "system",
    operation_id = "getAppInfo",
    responses(
        (status = 200, description = "Returns the status information about the Application", body = AppInfo),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    )
)]
pub async fn app_info_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<AppInfo>, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let status = app_status_or_default(secret_service);
  let setting_service = &state.app_service().setting_service();
  Ok(Json(AppInfo {
    version: setting_service.version(),
    status,
  }))
}

/// Request to setup the application in authenticated mode
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({}))]
pub struct SetupRequest {
  // Empty request - always sets up in authenticated mode
}

/// Response containing the updated application status after setup
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "resource-admin"
}))]
pub struct SetupResponse {
  /// New application status after setup
  /// - resource-admin: When setup in authenticated mode
  /// - ready: When setup in non-authenticated mode
  pub status: AppStatus,
}

impl IntoResponse for SetupResponse {
  fn into_response(self) -> Response {
    (StatusCode::OK, Json(self)).into_response()
  }
}

#[utoipa::path(
    post,
    path = ENDPOINT_APP_SETUP,
    tag = "setup",
    operation_id = "setupApp",
    request_body = SetupRequest,
    responses(
        (status = 200, description = "Application setup successful", body = SetupResponse),
        (status = 400, description = "Application is already setup", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Failed to register with auth server",
                 "type": "internal_server_error",
                 "code": "auth_service_error"
             }
         })
        )
    )
)]
pub async fn setup_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(_request), _): WithRejection<Json<SetupRequest>, ApiError>,
) -> Result<SetupResponse, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();
  let status = app_status_or_default(secret_service);
  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }
  // Always setup in authenticated mode
  let setting_service = &state.app_service().setting_service();
  let server_host = setting_service.host();
  let is_loopback =
    server_host == "localhost" || server_host == "127.0.0.1" || server_host == "0.0.0.0";
  let hosts = if is_loopback {
    vec!["localhost", "127.0.0.1", "0.0.0.0"]
  } else {
    vec![server_host.as_str()]
  };
  let scheme = setting_service.scheme();
  let port = setting_service.port();
  let callback_path = setting_service.login_callback_path();
  let redirect_uris = hosts
    .into_iter()
    .map(|host| format!("{scheme}://{host}:{port}{callback_path}"))
    .collect::<Vec<String>>();
  let app_reg_info = auth_service.register_client(redirect_uris).await?;
  secret_service.set_app_reg_info(&app_reg_info)?;
  secret_service.set_app_status(&AppStatus::ResourceAdmin)?;
  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}

/// Response to the ping endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PingResponse {
  /// always returns "pong"
  pub message: String,
}

/// Simple health check endpoint
#[utoipa::path(
    get,
    path = ENDPOINT_PING,
    tag = "system",
    operation_id = "pingServer",
    responses(
        (status = 200, description = "Server is healthy", 
         body = PingResponse,
         content_type = "application/json",
         example = json!({"message": "pong"})
        )
    )
)]
pub async fn ping_handler() -> Json<PingResponse> {
  Json(PingResponse {
    message: "pong".to_string(),
  })
}

#[cfg(test)]
mod tests {
  use crate::{
    app_info_handler, setup_handler, test_utils::TEST_ENDPOINT_APP_INFO, AppInfo, SetupRequest,
  };
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
    }
  )]
  #[case(
    SecretServiceStub::new().with_app_status(&AppStatus::Setup),
    AppInfo {
      version: "0.0.0".to_string(),
      status: AppStatus::Setup,
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
      .route(TEST_ENDPOINT_APP_INFO, get(app_info_handler))
      .with_state(state);
    let resp = router
      .oneshot(Request::get(TEST_ENDPOINT_APP_INFO).body(Body::empty())?)
      .await?;
    assert_eq!(StatusCode::OK, resp.status());
    let value = resp.json::<AppInfo>().await?;
    assert_eq!(expected, value);
    Ok(())
  }

  #[rstest]
  #[case(
      SecretServiceStub::new().with_app_status(&AppStatus::Ready),
      SetupRequest {},
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

    assert_eq!(StatusCode::BAD_REQUEST, resp.status());
    let body = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "app is already setup",
          "code": "app_service_error-already_setup",
          "type": "invalid_request_error",
        }
      }},
      body
    );

    let secret_service = app_service.secret_service();
    assert_eq!(AppStatus::Ready, secret_service.app_status().unwrap());
    let app_reg_info = secret_service.app_reg_info().unwrap();
    assert!(app_reg_info.is_none());
    Ok(())
  }

  #[rstest]
  #[case(
      SecretServiceStub::new(),
      SetupRequest {},
      AppStatus::ResourceAdmin,
  )]
  #[case(
      SecretServiceStub::new().with_app_status(&AppStatus::Setup),
      SetupRequest {},
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
      .times(1)
      .return_once(|_redirect_uris| {
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
    let app_reg_info = secret_service.app_reg_info().unwrap();
    assert!(app_reg_info.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_register_resource_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Setup);
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .times(1)
      .return_once(|_redirect_uris| {
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
          .body(Body::from(serde_json::to_string(&SetupRequest {})?))?,
      )
      .await?;

    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
    let body = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "error connecting to internal service: \u{2068}failed to register as resource server\u{2069}",
          "code": "reqwest_error",
          "type": "internal_server_error",
        }
      }},
      body
    );
    Ok(())
  }

  #[rstest]
  #[case(
    r#"{"invalid": true,}"#,
    "failed to parse the request body as JSON, error: \u{2068}Failed to parse the request body as JSON: trailing comma at line 1 column 18\u{2069}"
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
    assert_eq!(StatusCode::BAD_REQUEST, resp.status());
    let body = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": expected_error,
          "type": "invalid_request_error",
          "code": "json_rejection_error"
        }
      }},
      body
    );
    Ok(())
  }
}
