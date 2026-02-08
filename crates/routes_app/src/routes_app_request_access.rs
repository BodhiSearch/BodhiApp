use crate::{LoginError, ENDPOINT_APPS_REQUEST_ACCESS};
use axum::{extract::State, Json};
use objs::{ApiError, API_TAG_AUTH};
use server_core::RouterState;
use services::{
  db::AppClientToolsetConfigRow, AppAccessRequest, AppAccessResponse, AppClientToolset,
  SecretServiceExt,
};
use std::sync::Arc;

/// Request access for an app client to this resource server
#[utoipa::path(
    post,
    path = ENDPOINT_APPS_REQUEST_ACCESS,
    tag = API_TAG_AUTH,
    operation_id = "requestAccess",
    summary = "Request Resource Access",
    description = "Requests access permissions for an application client to access this resource server's protected resources. Supports caching via optional version parameter.",
    request_body(
        content = AppAccessRequest,
        description = "Application client requesting access",
        example = json!({
            "app_client_id": "my_app_client_123",
            "toolset_scope_ids": ["uuid-for-toolset-scope-1"],
            "version": "v1.0.0"
        })
    ),
    responses(
        (status = 200, description = "Access granted successfully", body = AppAccessResponse,
         example = json!({
             "scope": "scope_resource_bodhi-server",
             "toolsets": [{"id": "builtin-exa-web-search", "scope": "scope_toolset-builtin-exa-web-search"}],
             "app_client_config_version": "v1.0.0"
         })),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn request_access_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<AppAccessRequest>,
) -> Result<Json<AppAccessResponse>, ApiError> {
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();
  let db_service = app_service.db_service();

  // Get app registration info
  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  // Check cache if version is provided
  if let Ok(Some(cached)) = db_service
    .get_app_client_toolset_config(&request.app_client_id)
    .await
  {
    // Require both versions to be Some and matching for cache hit
    if let (Some(ref version), Some(ref cached_version)) =
      (&request.version, &cached.config_version)
    {
      if cached_version == version {
        // Version matches - check if all requested scope_ids are already added to resource-client
        let toolsets: Vec<AppClientToolset> =
          serde_json::from_str(&cached.toolsets_json).unwrap_or_default();

        let all_scopes_added = if let Some(ref requested_scope_ids) = request.toolset_scope_ids {
          // Check if all requested scope_ids have added_to_resource_client=true
          requested_scope_ids.iter().all(|requested_id| {
            toolsets
              .iter()
              .any(|t| &t.scope_id == requested_id && t.added_to_resource_client == Some(true))
          })
        } else {
          // No toolset_scope_ids requested - cache hit
          true
        };

        if all_scopes_added {
          // Full cache hit - return cached data
          return Ok(Json(AppAccessResponse {
            scope: cached.resource_scope,
            toolsets,
            app_client_config_version: cached.config_version.clone(),
          }));
        }
      }
    }
  }

  // Cache miss or scope_ids not yet added - call auth server with toolset_scope_ids
  let auth_response = auth_service
    .request_access(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      &request.app_client_id,
      request.toolset_scope_ids.clone(),
    )
    .await?;

  // Log if version mismatch or None cases
  match (&request.version, &auth_response.app_client_config_version) {
    (Some(req_ver), Some(resp_ver)) if req_ver != resp_ver => {
      tracing::warn!(
        "App client {} sent version {} but auth server returned version {}",
        request.app_client_id,
        req_ver,
        resp_ver
      );
    }
    (Some(_), None) | (None, Some(_)) | (None, None) => {
      tracing::warn!(
        "App client {} does not have config versioning configured properly",
        request.app_client_id
      );
    }
    _ => {} // versions match, no log needed
  }

  // Store/update cache
  let toolsets_json =
    serde_json::to_string(&auth_response.toolsets).unwrap_or_else(|_| "[]".to_string());
  let now = db_service.now().timestamp();
  let config_row = AppClientToolsetConfigRow {
    id: 0, // Will be set by DB
    app_client_id: request.app_client_id.clone(),
    config_version: auth_response.app_client_config_version.clone(),
    toolsets_json,
    resource_scope: auth_response.scope.clone(),
    created_at: now,
    updated_at: now,
  };

  if let Err(e) = db_service
    .upsert_app_client_toolset_config(&config_row)
    .await
  {
    tracing::warn!(
      "Failed to cache app client toolset config for {}: {}",
      request.app_client_id,
      e
    );
    // Don't fail the request if caching fails
  }

  Ok(Json(AppAccessResponse {
    scope: auth_response.scope,
    toolsets: auth_response.toolsets,
    app_client_config_version: auth_response.app_client_config_version,
  }))
}

#[cfg(test)]
mod tests {
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
}
