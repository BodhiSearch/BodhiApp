use crate::{
  app_info_handler, setup_handler, test_utils::TEST_ENDPOINT_APP_INFO, AppInfo, SetupRequest,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{get, post},
  Router,
};

use objs::ReqwestError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  test_utils::{AppServiceStubBuilder, SecretServiceStub, SettingServiceStub},
  AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService, SecretServiceExt,
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[anyhow_trace]
#[rstest]
#[case(
  SecretServiceStub::new(),
  AppInfo {
    version: "0.0.0".to_string(),
    commit_sha: "test-sha".to_string(),
    status: AppStatus::Setup,
  }
)]
#[case(
  SecretServiceStub::new().with_app_status(&AppStatus::Setup),
  AppInfo {
    version: "0.0.0".to_string(),
    commit_sha: "test-sha".to_string(),
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
    .build()
    .await?;
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

#[anyhow_trace]
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
  #[case] secret_service: SecretServiceStub,
  #[case] payload: SetupRequest,
) -> anyhow::Result<()> {
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(Arc::new(MockAuthService::new()))
      .build()
      .await?,
  );
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/setup", post(setup_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(payload)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!(
    "app_service_error-already_setup",
    body["error"]["code"].as_str().unwrap()
  );

  let secret_service = app_service.secret_service();
  assert_eq!(AppStatus::Ready, secret_service.app_status()?);
  let app_reg_info = secret_service.app_reg_info()?;
  assert!(app_reg_info.is_none());
  Ok(())
}

#[anyhow_trace]
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
        scope: "scope_client_id".to_string(),
      })
    });
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(Arc::new(mock_auth_service))
      .build()
      .await?,
  );
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/setup", post(setup_handler))
    .with_state(state);

  let response = router
    .oneshot(Request::post("/setup").json(request)?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let secret_service = app_service.secret_service();
  assert_eq!(expected_status, secret_service.app_status()?);
  let app_reg_info = secret_service.app_reg_info()?;
  assert!(app_reg_info.is_some());
  Ok(())
}

#[anyhow_trace]
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
        scope: "scope_client_id".to_string(),
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
      .build()
      .await?,
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
        .header("Host", "localhost:1135")
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let secret_service = app_service.secret_service();
  assert_eq!(AppStatus::ResourceAdmin, secret_service.app_status()?);
  let app_reg_info = secret_service.app_reg_info()?;
  assert!(app_reg_info.is_some());
  Ok(())
}

#[anyhow_trace]
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
        scope: "scope_client_id".to_string(),
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
      .build()
      .await?,
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
        .header("Host", "192.168.1.100:1135")
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let secret_service = app_service.secret_service();
  assert_eq!(AppStatus::ResourceAdmin, secret_service.app_status()?);
  let app_reg_info = secret_service.app_reg_info()?;
  assert!(app_reg_info.is_some());
  Ok(())
}

#[anyhow_trace]
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
        && redirect_uris.contains(&"https://my-bodhi.example.com:8443/ui/auth/callback".to_string())
    })
    .return_once(|_name, _description, _redirect_uris| {
      Ok(AppRegInfo {
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
        scope: "scope_client_id".to_string(),
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
      .build()
      .await?,
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
        .header("Host", "192.168.1.100:1135") // This should be ignored due to explicit config
        .json(request)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let secret_service = app_service.secret_service();
  assert_eq!(AppStatus::ResourceAdmin, secret_service.app_status()?);
  let app_reg_info = secret_service.app_reg_info()?;
  assert!(app_reg_info.is_some());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_register_resource_error() -> anyhow::Result<()> {
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
      .build()
      .await?,
  );
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/setup", post(setup_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(SetupRequest {
      name: "Test Server Name".to_string(),
      description: Some("Test description".to_string()),
    })?)
    .await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!("reqwest_error", body["error"]["code"].as_str().unwrap());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case(r#"{"invalid": true,}"#)]
#[tokio::test]
async fn test_setup_handler_bad_request(#[case] body: &str) -> anyhow::Result<()> {
  let app_service = Arc::new(AppServiceStubBuilder::default().build().await?);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/setup", post(setup_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json_str(body)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  let body = resp.json::<Value>().await?;
  assert_eq!(
    "json_rejection_error",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_handler_validation_error() -> anyhow::Result<()> {
  let mock_auth_service = MockAuthService::default();
  // No expectation needed as validation should fail before calling auth service

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::new()))
      .auth_service(Arc::new(mock_auth_service))
      .build()
      .await?,
  );
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/setup", post(setup_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/setup").json(SetupRequest {
      name: "Short".to_string(), // Less than 10 characters
      description: Some("Test description".to_string()),
    })?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  Ok(())
}
