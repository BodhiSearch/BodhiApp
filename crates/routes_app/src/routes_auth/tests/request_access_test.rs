use crate::request_access_handler;
use axum::{
  http::{status::StatusCode, Request},
  routing::post,
  Router,
};
use mockito::{Matcher, Server};
use objs::test_utils::temp_bodhi_home;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  db::AppClientToolsetConfigRow,
  test_utils::{test_auth_service, AppServiceStubBuilder, SecretServiceStub},
  AppAccessResponse, AppRegInfo, AppService, AppStatus,
};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
async fn test_request_access_handler_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let app_client_id = "test_app_client_id";
  let expected_scope = "scope_resource_test-resource-server";

  // Mock auth server
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint for client credentials
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock request-access endpoint
  let access_mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
    .match_header("Authorization", "Bearer test_access_token")
    .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
    .with_status(200)
    .with_body(
      json!({
        "scope": expected_scope,
        "toolsets": [{
          "id": "builtin-exa-web-search",
          "scope": "scope_toolset-builtin-exa-web-search",
          "scope_id": "test-scope-uuid-123",
          "added_to_resource_client": true
        }],
        "app_client_config_version": "v1.0.0"
      })
      .to_string(),
    )
    .create();

  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let auth_service = Arc::new(test_auth_service(&url));
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .auth_service(auth_service)
    .with_db_service()
    .await
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/request-access", post(request_access_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/auth/request-access").json(json! {{
      "app_client_id": app_client_id
    }})?)
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: AppAccessResponse = resp.json().await?;
  assert_eq!(expected_scope, body.scope);
  assert_eq!(Some("v1.0.0".to_string()), body.app_client_config_version);

  token_mock.assert();
  access_mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_request_access_handler_none_version_cache_miss(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_client_id = "test_app_client_id";
  let expected_scope = "scope_resource_test-resource-server";

  // Mock auth server
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock request-access endpoint
  let access_mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
    .match_header("Authorization", "Bearer test_access_token")
    .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
    .with_status(200)
    .with_body(
      json!({
        "scope": expected_scope,
        "toolsets": [{
          "id": "builtin-exa-web-search",
          "scope": "scope_toolset-builtin-exa-web-search",
          "scope_id": "test-scope-uuid-123",
          "added_to_resource_client": true
        }],
        "app_client_config_version": "v2.0.0"
      })
      .to_string(),
    )
    .create();

  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let auth_service = Arc::new(test_auth_service(&url));
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .auth_service(auth_service)
    .with_db_service()
    .await
    .build_session_service(dbfile.clone())
    .await
    .build()?;

  // Pre-populate cache with None version
  let db_service = app_service.db_service();
  let now = db_service.now().timestamp();
  db_service
    .upsert_app_client_toolset_config(&AppClientToolsetConfigRow {
      id: 0,
      app_client_id: app_client_id.to_string(),
      config_version: None,
      toolsets_json: "[]".to_string(),
      resource_scope: "old_scope".to_string(),
      created_at: now,
      updated_at: now,
    })
    .await?;

  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/request-access", post(request_access_handler))
    .with_state(state);

  // Client sends version but DB has None - should be cache miss
  let resp = router
    .oneshot(Request::post("/auth/request-access").json(json! {{
      "app_client_id": app_client_id,
      "version": "v1.0.0"
    }})?)
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: AppAccessResponse = resp.json().await?;
  assert_eq!(expected_scope, body.scope);
  assert_eq!(Some("v2.0.0".to_string()), body.app_client_config_version);

  token_mock.assert();
  access_mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_request_access_handler_auth_returns_none_version(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_client_id = "test_app_client_id";
  let expected_scope = "scope_resource_test-resource-server";

  // Mock auth server
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock request-access endpoint without version field
  let access_mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
    .match_header("Authorization", "Bearer test_access_token")
    .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
    .with_status(200)
    .with_body(
      json!({
        "scope": expected_scope,
        "toolsets": [{
          "id": "builtin-exa-web-search",
          "scope": "scope_toolset-builtin-exa-web-search",
          "scope_id": "test-scope-uuid-123",
          "added_to_resource_client": true
        }]
      })
      .to_string(),
    )
    .create();

  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let auth_service = Arc::new(test_auth_service(&url));
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .auth_service(auth_service)
    .with_db_service()
    .await
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/request-access", post(request_access_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/auth/request-access").json(json! {{
      "app_client_id": app_client_id
    }})?)
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: AppAccessResponse = resp.json().await?;
  assert_eq!(expected_scope, body.scope);
  assert_eq!(None, body.app_client_config_version);

  token_mock.assert();
  access_mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_request_access_handler_no_client_credentials(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Ready); // No app_reg_info set
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .with_db_service()
    .await
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/request-access", post(request_access_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/auth/request-access").json(json! {{
      "app_client_id": "test_app_client_id"
    }})?)
    .await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  let error = resp.json::<Value>().await?;
  let expected_message = "Application is not registered. Please register the application first.";
  assert_eq!(
    json! {{
      "error": {
        "message": expected_message,
        "code": "login_error-app_reg_info_not_found",
        "type": "invalid_app_state"
      }
    }},
    error
  );
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_request_access_handler_auth_service_error(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let app_client_id = "invalid_app_client_id";

  // Mock auth server with error response
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint for client credentials
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock request-access endpoint with error
  let access_mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
    .with_status(400)
    .with_body(json!({"error": "app_client_not_found"}).to_string())
    .create();

  let dbfile = temp_bodhi_home.path().join("test.db");
  let secret_service = SecretServiceStub::new()
    .with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    })
    .with_app_status(&AppStatus::Ready);
  let auth_service = Arc::new(test_auth_service(&url));
  let app_service = AppServiceStubBuilder::default()
    .secret_service(Arc::new(secret_service))
    .auth_service(auth_service)
    .with_db_service()
    .await
    .build_session_service(dbfile)
    .await
    .build()?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/auth/request-access", post(request_access_handler))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/auth/request-access").json(json! {{
      "app_client_id": app_client_id
    }})?)
    .await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  let error = resp.json::<Value>().await?;
  assert_eq!(
    json! {{
      "error": {
        "message": "Authentication service API error (status 0): app_client_not_found.",
        "code": "auth_service_error-auth_service_api_error",
        "type": "internal_server_error",
        "param": {
          "status": "0",
          "body": "app_client_not_found"
        }
      }
    }},
    error
  );

  token_mock.assert();
  access_mock.assert();
  Ok(())
}
