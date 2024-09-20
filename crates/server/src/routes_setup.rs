use super::RouterStateFn;
use crate::{BadRequestError, HttpError, HttpErrorBuilder};
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use services::{
  set_secret, AuthServiceError, SecretServiceError, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_APP_STATUS,
};
use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppServiceError {
  #[error("app is already set up")]
  AlreadySetup,
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}

impl From<AppServiceError> for HttpError {
  fn from(value: AppServiceError) -> Self {
    match value {
      value @ AppServiceError::AlreadySetup => HttpErrorBuilder::default()
        .bad_request(&value.to_string())
        .build()
        .unwrap(),
      AppServiceError::SecretServiceError(err) => err.into(),
      AppServiceError::AuthServiceError(err) => err.into(),
    }
  }
}

impl IntoResponse for AppServiceError {
  fn into_response(self) -> Response {
    HttpError::from(self).into_response()
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct AppInfo {
  version: String,
  status: String,
  authz: bool,
}

pub(crate) async fn app_info_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<AppInfo>, HttpError> {
  let secret_service = &state.app_service().secret_service();
  let authz = secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or(Some("true".to_string()))
    .unwrap_or("true".to_string());
  let status = secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or(Some("setup".to_string()))
    .unwrap_or("setup".to_string());
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
  status: String,
}

impl IntoResponse for SetupResponse {
  fn into_response(self) -> Response {
    (StatusCode::OK, Json(self)).into_response()
  }
}

pub async fn setup_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, BadRequestError>,
) -> Result<SetupResponse, AppServiceError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();

  let status = secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or(Some("setup".to_string()))
    .unwrap_or("setup".to_string());

  if status != "setup" {
    return Err(AppServiceError::AlreadySetup);
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
      status: "resource-admin".to_string(),
    })
  } else {
    secret_service.set_secret_string(KEY_APP_AUTHZ, "false")?;
    secret_service.set_secret_string(KEY_APP_STATUS, "ready")?;
    Ok(SetupResponse {
      status: "ready".to_string(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::{MockSharedContext, ResponseTestExt};
  use crate::{ErrorBody, RouterState};
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
  };
  use jsonwebtoken::Algorithm;
  use objs::AppRegInfo;
  use rstest::rstest;
  use services::{
    get_secret,
    test_utils::{AppServiceStubBuilder, SecretServiceStub},
    AppServiceFn, MockAuthService,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  #[rstest]
  #[case(
    SecretServiceStub::new(),
    AppInfo {
      version: "0.0.0".to_string(),
      status: "setup".to_string(),
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
      status: "setup".to_string(),
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
      status: "setup".to_string(),
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
    let state = Arc::new(RouterState::new(
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
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "ready".to_string(),
      }),
      SetupRequest { authz: true },
      StatusCode::BAD_REQUEST,
      "app is already set up".to_string(),
  )]
  #[tokio::test]
  async fn test_setup_handler_error(
    #[case] secret_service: SecretServiceStub,
    #[case] payload: SetupRequest,
    #[case] expected_status: StatusCode,
    #[case] expected_error: String,
  ) -> anyhow::Result<()> {
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(MockAuthService::new()))
        .build()?,
    );
    let state = Arc::new(RouterState::new(
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

    assert_eq!(resp.status(), expected_status);
    let body: ErrorBody = resp.json().await?;
    assert_eq!(body.message, expected_error);

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
      "ready",
  )]
  #[case(
      SecretServiceStub::new(),
      SetupRequest { authz: true },
      "resource-admin",
  )]
  #[case(
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "setup".to_string(),
      }),
      SetupRequest { authz: false },
      "ready",
  )]
  #[case(
      SecretServiceStub::with_map(maplit::hashmap! {
          KEY_APP_STATUS.to_string() => "setup".to_string(),
      }),
      SetupRequest { authz: true },
      "resource-admin",
  )]
  #[tokio::test]
  async fn test_setup_handler_success(
    #[case] secret_service: SecretServiceStub,
    #[case] request: SetupRequest,
    #[case] expected_status: &str,
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
    let state = Arc::new(RouterState::new(
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
  async fn test_setup_handler_register_resource_error() -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
        KEY_APP_STATUS.to_string() => "setup".to_string(),
    });
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_register_client()
      .once()
      .returning(|_redirect_uris| {
        Err(AuthServiceError::Reqwest(
          "failed to register as resource server".to_string(),
        ))
      });
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .secret_service(Arc::new(secret_service))
        .auth_service(Arc::new(mock_auth_service))
        .build()?,
    );
    let state = Arc::new(RouterState::new(
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
    let body: ErrorBody = resp.json().await?;
    assert_eq!("failed to register as resource server", body.message);
    Ok(())
  }

  #[rstest]
  #[case(
    r#"{"authz": "invalid"}"#,
    "We could not parse the JSON body of your request: JSONDataError: Failed to deserialize the JSON body into the target type"
  )]
  #[case(
    r#"{}"#,
    "We could not parse the JSON body of your request: JSONDataError: Failed to deserialize the JSON body into the target type"
  )]
  #[case(
    r#"{"authz": true,}"#,
    "We could not parse the JSON body of your request: JSONSyntaxError: Failed to parse the request body as JSON"
  )]
  #[tokio::test]
  async fn test_setup_handler_bad_request(
    #[case] body: &str,
    #[case] expected_error: &str,
  ) -> anyhow::Result<()> {
    let app_service = Arc::new(AppServiceStubBuilder::default().build()?);
    let state = Arc::new(RouterState::new(
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
    let body: ErrorBody = resp.json().await?;
    assert_eq!(
      ErrorBody {
        message: expected_error.to_string(),
        r#type: "invalid_request_error".to_string(),
        param: None,
        code: Some("invalid_value".to_string())
      },
      body
    );
    Ok(())
  }
}
