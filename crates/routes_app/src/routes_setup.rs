use crate::utils::extract_request_host;
use crate::{ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_HEALTH, ENDPOINT_PING};
use auth_middleware::app_status_or_default;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, ErrorType, OpenAIApiError, API_TAG_SETUP, API_TAG_SYSTEM};
use serde::{Deserialize, Serialize};
use serde_json::json;
use server_core::RouterState;
use services::{
  AppStatus, AuthServiceError, SecretServiceError, SecretServiceExt, LOGIN_CALLBACK_PATH,
};
use std::sync::Arc;
use utoipa::ToSchema;

pub const LOOPBACK_HOSTS: &[&str] = &["localhost", "127.0.0.1", "0.0.0.0"];

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
  /// Application version number (semantic versioning)
  #[schema(example = "0.1.0")]
  pub version: String,
  /// Current application setup and operational status
  #[schema(example = "ready")]
  pub status: AppStatus,
}

#[utoipa::path(
    get,
    path = ENDPOINT_APP_INFO,
    tag = API_TAG_SYSTEM,
    operation_id = "getAppInfo",
    summary = "Get Application Information",
    description = "Retrieves current application version and status information including setup state",
    responses(
        (status = 200, description = "Application information retrieved successfully", body = AppInfo,
         example = json!({
             "version": "0.1.0",
             "status": "ready"
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Failed to retrieve application status",
                 "type": "internal_server_error",
                 "code": "system_error"
             }
         }))
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
#[schema(example = json!({
  "name": "My Bodhi Server",
  "description": "My personal AI server"
}))]
pub struct SetupRequest {
  /// Server name for identification (minimum 10 characters)
  #[schema(min_length = 10, max_length = 100, example = "My Bodhi Server")]
  pub name: String,
  /// Optional description of the server's purpose
  #[schema(max_length = 500, example = "My personal AI server")]
  pub description: Option<String>,
}

/// Response containing the updated application status after setup
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "resource-admin"
}))]
pub struct SetupResponse {
  /// New application status after successful setup
  #[schema(example = "resource-admin")]
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
    tag = API_TAG_SETUP,
    operation_id = "setupApp",
    summary = "Setup Application",
    description = "Initializes the application with authentication configuration and registers with the auth server",
    request_body(
        content = SetupRequest,
        description = "Application setup configuration",
        example = json!({
            "name": "My Bodhi Server",
            "description": "My personal AI server"
        })
    ),
    responses(
        (status = 200, description = "Application setup completed successfully", body = SetupResponse,
         example = json!({
             "status": "resource-admin"
         })),
        (status = 400, description = "Invalid request or application already setup", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Server name must be at least 10 characters long",
                 "type": "invalid_request_error",
                 "code": "validation_error"
             }
         })),
        (status = 500, description = "Internal server error during setup", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Failed to register with auth server",
                 "type": "internal_server_error",
                 "code": "auth_service_error"
             }
         }))
    )
)]
pub async fn setup_handler(
  headers: axum::http::HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, ApiError>,
) -> Result<SetupResponse, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();
  let status = app_status_or_default(secret_service);
  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }

  // Validate server name (minimum 10 characters)
  if request.name.len() < 10 {
    // TODO: localize this error message
    return Err(objs::BadRequestError::new(
      "Server name must be at least 10 characters long".to_string(),
    ))?;
  }
  let setting_service = &state.app_service().setting_service();
  let redirect_uris = if setting_service.get_public_host_explicit().is_some() {
    // Explicit configuration (including RunPod) - use only configured callback URL
    vec![setting_service.login_callback_url()]
  } else {
    // Local/network installation mode - build comprehensive redirect URI list
    let scheme = setting_service.public_scheme();
    let port = setting_service.public_port();
    let mut redirect_uris = Vec::new();

    // Always add all loopback hosts for local development
    for host in LOOPBACK_HOSTS {
      redirect_uris.push(format!(
        "{}://{}:{}{}",
        scheme, host, port, LOGIN_CALLBACK_PATH
      ));
    }

    // Add request host if it's not a loopback host (for network access)
    if let Some(request_host) = extract_request_host(&headers) {
      if !LOOPBACK_HOSTS.contains(&request_host.as_str()) {
        redirect_uris.push(format!(
          "{}://{}:{}{}",
          scheme, request_host, port, LOGIN_CALLBACK_PATH
        ));
      }
    }

    // Add server IP for future-proofing (even if current request is from loopback)
    if let Some(server_ip) = get_server_ip() {
      let server_uri = format!("{}://{}:{}{}", scheme, server_ip, port, LOGIN_CALLBACK_PATH);
      // Only add if not already present
      if !redirect_uris.contains(&server_uri) {
        redirect_uris.push(server_uri);
      }
    }

    redirect_uris
  };
  let app_reg_info = auth_service
    .register_client(
      request.name,
      request.description.unwrap_or_default(),
      redirect_uris,
    )
    .await?;
  secret_service.set_app_reg_info(&app_reg_info)?;
  secret_service.set_app_status(&AppStatus::ResourceAdmin)?;
  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}

/// Response to the ping endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "pong"
}))]
pub struct PingResponse {
  /// Simple ping response message
  #[schema(example = "pong")]
  pub message: String,
}

/// Simple connectivity check endpoint
#[utoipa::path(
    get,
    path = ENDPOINT_PING,
    tag = API_TAG_SYSTEM,
    operation_id = "pingServer",
    summary = "Ping Server",
    description = "Simple connectivity check to verify the server is responding",
    responses(
        (status = 200, description = "Server is responding normally", 
         body = PingResponse,
         content_type = "application/json",
         example = json!({"message": "pong"})
        )
    )
)]
#[tracing::instrument]
pub async fn ping_handler() -> Json<PingResponse> {
  tracing::info!("ping request received");
  Json(PingResponse {
    message: "pong".to_string(),
  })
}

/// Application health check endpoint
#[utoipa::path(
  get,
  path = ENDPOINT_HEALTH,
  tag = API_TAG_SYSTEM,
  operation_id = "healthCheck",
  summary = "Health Check",
  description = "Comprehensive health check to verify all application components are operational",
  responses(
      (status = 200, description = "Application is healthy and fully operational",
       body = PingResponse,
       content_type = "application/json",
       example = json!({"message": "pong"})
      )
  )
)]
#[tracing::instrument]
pub async fn health_handler() -> Json<PingResponse> {
  tracing::info!("health check request received");
  Json(PingResponse {
    message: "pong".to_string(),
  })
}

/// Get the server's local IP address for future-proofing redirect URIs
fn get_server_ip() -> Option<String> {
  use std::net::UdpSocket;

  // Try to get local IP by connecting to a remote address
  // This doesn't actually send data, just determines which local interface would be used
  if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
    if let Ok(_) = socket.connect("8.8.8.8:80") {
      if let Ok(local_addr) = socket.local_addr() {
        let ip = local_addr.ip();
        // Only return if it's not a loopback address
        if !ip.is_loopback() {
          return Some(ip.to_string());
        }
      }
    }
  }

  None
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

  use objs::{test_utils::setup_l10n, FluentLocalizationService, ReqwestError};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    test_utils::{AppServiceStubBuilder, SecretServiceStub, SettingServiceStub},
    AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService, SecretServiceExt,
  };
  use std::{collections::HashMap, sync::Arc};
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
      SetupRequest {
        name: "Test Server Name".to_string(),
        description: Some("Test description".to_string()),
      },
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
      SetupRequest {
        name: "Test Server Name".to_string(),
        description: Some("Test description".to_string()),
      },
      AppStatus::ResourceAdmin,
  )]
  #[case(
      SecretServiceStub::new().with_app_status(&AppStatus::Setup),
      SetupRequest {
        name: "Test Server Name".to_string(),
        description: Some("Test description".to_string()),
      },
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
      .return_once(|_name, _description, _redirect_uris| {
        Ok(AppRegInfo {
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
  async fn test_setup_handler_loopback_redirect_uris() -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Setup);

    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .times(1)
      .withf(|_name, _description, redirect_uris| {
        // Verify that all loopback redirect URIs are registered
        // Now there might be additional URIs (request host, server IP) so check >= 3
        redirect_uris.len() >= 3
          && redirect_uris.contains(&"http://localhost:1135/ui/auth/callback".to_string())
          && redirect_uris.contains(&"http://127.0.0.1:1135/ui/auth/callback".to_string())
          && redirect_uris.contains(&"http://0.0.0.0:1135/ui/auth/callback".to_string())
      })
      .return_once(|_name, _description, _redirect_uris| {
        Ok(AppRegInfo {
          client_id: "client_id".to_string(),
          client_secret: "client_secret".to_string(),
        })
      });

    // Configure with default settings (no explicit public host)
    let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
      (services::BODHI_SCHEME.to_string(), "http".to_string()),
      (services::BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (services::BODHI_PORT.to_string(), "1135".to_string()),
    ]));

    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .setting_service(Arc::new(setting_service))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let request = SetupRequest {
      name: "Test Server Name".to_string(),
      description: Some("Test description".to_string()),
    };

    let response = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .header("Host", "localhost:1135") // Add Host header
          .body(Body::from(serde_json::to_string(&request)?))?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let secret_service = app_service.secret_service();
    assert_eq!(
      AppStatus::ResourceAdmin,
      secret_service.app_status().unwrap()
    );
    let app_reg_info = secret_service.app_reg_info().unwrap();
    assert!(app_reg_info.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_network_ip_redirect_uris() -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Setup);

    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .times(1)
      .withf(|_name, _description, redirect_uris| {
        // Verify that all loopback hosts AND the network IP are registered
        redirect_uris.len() >= 4  // 3 loopback + 1 network IP (+ optional server IP)
          && redirect_uris.contains(&"http://localhost:1135/ui/auth/callback".to_string())
          && redirect_uris.contains(&"http://127.0.0.1:1135/ui/auth/callback".to_string())
          && redirect_uris.contains(&"http://0.0.0.0:1135/ui/auth/callback".to_string())
          && redirect_uris.contains(&"http://192.168.1.100:1135/ui/auth/callback".to_string())
      })
      .return_once(|_name, _description, _redirect_uris| {
        Ok(AppRegInfo {
          client_id: "client_id".to_string(),
          client_secret: "client_secret".to_string(),
        })
      });

    // Configure with default settings (no explicit public host)
    let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
      (services::BODHI_SCHEME.to_string(), "http".to_string()),
      (services::BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (services::BODHI_PORT.to_string(), "1135".to_string()),
    ]));

    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .setting_service(Arc::new(setting_service))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let request = SetupRequest {
      name: "Test Server Name".to_string(),
      description: Some("Test description".to_string()),
    };

    let response = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .header("Host", "192.168.1.100:1135") // Network IP Host header
          .body(Body::from(serde_json::to_string(&request)?))?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let secret_service = app_service.secret_service();
    assert_eq!(
      AppStatus::ResourceAdmin,
      secret_service.app_status().unwrap()
    );
    let app_reg_info = secret_service.app_reg_info().unwrap();
    assert!(app_reg_info.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_explicit_public_host_single_redirect_uri() -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Setup);

    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .times(1)
      .withf(|_name, _description, redirect_uris| {
        // When public host is explicitly set, should only register that one
        redirect_uris.len() == 1
          && redirect_uris
            .contains(&"https://my-bodhi.example.com:8443/ui/auth/callback".to_string())
      })
      .return_once(|_name, _description, _redirect_uris| {
        Ok(AppRegInfo {
          client_id: "client_id".to_string(),
          client_secret: "client_secret".to_string(),
        })
      });

    // Configure with explicit public host
    let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
      (
        services::BODHI_PUBLIC_SCHEME.to_string(),
        "https".to_string(),
      ),
      (
        services::BODHI_PUBLIC_HOST.to_string(),
        "my-bodhi.example.com".to_string(),
      ),
      (services::BODHI_PUBLIC_PORT.to_string(), "8443".to_string()),
    ]));

    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .setting_service(Arc::new(setting_service))
        .build()?,
    );
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/setup", post(setup_handler))
      .with_state(state);

    let request = SetupRequest {
      name: "Test Server Name".to_string(),
      description: Some("Test description".to_string()),
    };

    let response = router
      .oneshot(
        Request::post("/setup")
          .header("Content-Type", "application/json")
          .header("Host", "192.168.1.100:1135") // This should be ignored due to explicit config
          .body(Body::from(serde_json::to_string(&request)?))?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let secret_service = app_service.secret_service();
    assert_eq!(
      AppStatus::ResourceAdmin,
      secret_service.app_status().unwrap()
    );
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
      .return_once(|_name, _description, _redirect_uris| {
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
            name: "Test Server Name".to_string(),
            description: Some("Test description".to_string()),
          })?))?,
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

  #[rstest]
  #[tokio::test]
  async fn test_setup_handler_validation_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let mock_auth_service = MockAuthService::default();
    // No expectation needed as validation should fail before calling auth service

    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(SecretServiceStub::new()))
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
            name: "Short".to_string(), // Less than 10 characters
            description: Some("Test description".to_string()),
          })?))?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, resp.status());
    Ok(())
  }
}
