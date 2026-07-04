use crate::mcps::{
  mcp_proxy_handler, mcps_create, mcps_destroy, mcps_index, mcps_show, mcps_update,
};
use crate::test_utils::RequestAuthContextExt;
use crate::test_utils::{build_mcp_test_state, fixed_dt};
use crate::BodhiErrorResponse;
use anyhow_trace::anyhow_trace;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use server_core::test_utils::ResponseTestExt;
use services::AuthContext;
use services::McpError;
use services::MockMcpService;
use services::ResourceRole;
use services::{
  Mcp, McpAuthParamInput, McpAuthParamType, McpAuthType, McpRequest, McpWithServerEntity,
};
use services::{McpGrant, ModelGrant, TokenGrants, TokenGrantsV1, TokenScope};
use tower::ServiceExt;

#[fixture]
fn test_mcp_entity() -> McpWithServerEntity {
  let now = fixed_dt();
  McpWithServerEntity {
    id: "mcp-uuid-1".to_string(),
    user_id: "user123".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    slug: "deepwiki".to_string(),
    name: "DeepWiki MCP".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    created_at: now,
    updated_at: now,
    server_url: "https://mcp.deepwiki.com/mcp".to_string(),
    server_name: "DeepWiki".to_string(),
    server_enabled: true,
  }
}

async fn test_router_for_crud(mock_mcp_service: MockMcpService) -> anyhow::Result<Router> {
  let state = build_mcp_test_state(mock_mcp_service).await?;
  Ok(
    Router::new()
      .route("/mcps", get(mcps_index))
      .route("/mcps", post(mcps_create))
      .route("/mcps/{id}", get(mcps_show))
      .route("/mcps/{id}", put(mcps_update))
      .route("/mcps/{id}", delete(mcps_destroy))
      .with_state(state),
  )
}

// ============================================================================
// GET /mcps - API-token grant filter (shared with /apps/mcps)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcps_index_api_token_grant_filters_list(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let granted = test_mcp_entity.clone(); // id "mcp-uuid-1"
  let mut other = test_mcp_entity.clone();
  other.id = "mcp-uuid-2".to_string();
  other.slug = "other".to_string();

  let mut mock = MockMcpService::new();
  mock
    .expect_list()
    .returning(move |_, _| Ok(vec![granted.clone(), other.clone()]));
  let app = test_router_for_crud(mock).await?;

  // Grant only "mcp-uuid-1" for connect, mcps_list off → only that instance is listed.
  let token = AuthContext::test_api_token_with_grants(
    "user",
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: false,
      models: ModelGrant::All,
      mcps_list: false,
      mcps: McpGrant::Specific {
        ids: vec!["mcp-uuid-1".to_string()],
      },
    }),
  );

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(token),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let ids: Vec<&str> = body["mcps"]
    .as_array()
    .unwrap()
    .iter()
    .map(|m| m["id"].as_str().unwrap())
    .collect();
  assert_eq!(vec!["mcp-uuid-1"], ids);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcps_index_external_app_grant_filters_list(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  // An approved app's grants flow through the SAME AccessPolicy as API tokens —
  // no bespoke access-request re-fetch. Owner granted only "mcp-uuid-1" (via
  // mcps_access), mcps_list off ⇒ only that instance is listed.
  let granted = test_mcp_entity.clone(); // id "mcp-uuid-1"
  let mut other = test_mcp_entity.clone();
  other.id = "mcp-uuid-2".to_string();
  other.slug = "other".to_string();

  let mut mock = MockMcpService::new();
  mock
    .expect_list()
    .returning(move |_, _| Ok(vec![granted.clone(), other.clone()]));
  let app = test_router_for_crud(mock).await?;

  let approved = services::ApprovedResources::V1(services::ApprovedResourcesV1 {
    models_list: false,
    models_access: ModelGrant::All,
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific {
      ids: vec!["mcp-uuid-1".to_string()],
    },
  });
  let ctx = AuthContext::test_external_app("user", services::UserScope::User, "app", Some("ar"))
    .with_external_app_grants(approved);

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(ctx),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let ids: Vec<&str> = body["mcps"]
    .as_array()
    .unwrap()
    .iter()
    .map(|m| m["id"].as_str().unwrap())
    .collect();
  assert_eq!(vec!["mcp-uuid-1"], ids);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcps_index_list_toggle_on_annotates_access(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  // M2: mcps_list on ⇒ both instances listed; only the connect-granted one is `access:true`.
  let granted = test_mcp_entity.clone(); // id "mcp-uuid-1"
  let mut other = test_mcp_entity.clone();
  other.id = "mcp-uuid-2".to_string();
  other.slug = "other".to_string();

  let mut mock = MockMcpService::new();
  mock
    .expect_list()
    .returning(move |_, _| Ok(vec![granted.clone(), other.clone()]));
  let app = test_router_for_crud(mock).await?;

  let token = AuthContext::test_api_token_with_grants(
    "user",
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: false,
      models: ModelGrant::All,
      mcps_list: true,
      mcps: McpGrant::Specific {
        ids: vec!["mcp-uuid-1".to_string()],
      },
    }),
  );

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(token),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let access: Vec<(&str, bool)> = body["mcps"]
    .as_array()
    .unwrap()
    .iter()
    .map(|m| (m["id"].as_str().unwrap(), m["access"].as_bool().unwrap()))
    .collect();
  assert_eq!(vec![("mcp-uuid-1", true), ("mcp-uuid-2", false)], access);
  Ok(())
}

// ============================================================================
// GET /mcps/{id} - existence-hiding for scoped tokens (N6)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcps_show_scoped_token_hides_ungranted(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  // `get` is only reached for the granted (listable) id; the ungranted id is
  // 404'd by the listability check before any service call.
  let granted = test_mcp_entity.clone(); // id "mcp-uuid-1"
  let mut mock = MockMcpService::new();
  mock
    .expect_get()
    .times(0..)
    .returning(move |_, _, _| Ok(Some(granted.clone())));
  let app = test_router_for_crud(mock).await?;

  let token = AuthContext::test_api_token_with_grants(
    "user",
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: false,
      models: ModelGrant::All,
      mcps_list: false,
      mcps: McpGrant::Specific {
        ids: vec!["mcp-uuid-1".to_string()],
      },
    }),
  );

  // Existing-but-ungranted instance → 404 (existence not revealed, not 403).
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/mcps/mcp-uuid-2")
        .body(Body::empty())?
        .with_auth_context(token.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  // Granted instance → 200.
  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/mcps/mcp-uuid-1")
        .body(Body::empty())?
        .with_auth_context(token),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// POST /apps/mcps/{id}/mcp - proxy grant gate (C1)
// ============================================================================

/// Proxy router with `get` stubbed to `None` so a granted invocation that
/// clears the grant gate surfaces the absent upstream as a clean 404 (never a
/// gate 403). `times(0..)` because the ungranted path never reaches `get`.
async fn proxy_router() -> anyhow::Result<Router> {
  let mut mock = MockMcpService::new();
  mock.expect_get().times(0..).returning(|_, _, _| Ok(None));
  let state = build_mcp_test_state(mock).await?;
  Ok(
    Router::new()
      .route("/bodhi/v1/apps/mcps/{id}/mcp", post(mcp_proxy_handler))
      .with_state(state),
  )
}

/// API token granting MCP connect only on "mcp-uuid-1".
fn proxy_api_token() -> AuthContext {
  AuthContext::test_api_token_with_grants(
    "user",
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: false,
      models: ModelGrant::All,
      mcps_list: false,
      mcps: McpGrant::Specific {
        ids: vec!["mcp-uuid-1".to_string()],
      },
    }),
  )
}

/// Approved external app whose grants ride the SAME AccessPolicy — connect only
/// on "mcp-uuid-1".
fn proxy_external_app() -> AuthContext {
  let approved = services::ApprovedResources::V1(services::ApprovedResourcesV1 {
    models_list: false,
    models_access: ModelGrant::All,
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific {
      ids: vec!["mcp-uuid-1".to_string()],
    },
  });
  AuthContext::test_external_app("user", services::UserScope::User, "app", Some("ar"))
    .with_external_app_grants(approved)
}

#[rstest]
#[case::api_token(proxy_api_token())]
#[case::external_app(proxy_external_app())]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_proxy_enforces_mcp_connect_grant(#[case] ctx: AuthContext) -> anyhow::Result<()> {
  // Ungranted instance → blocked at the grant gate with mcp_forbidden, before any service call.
  let deny_app = proxy_router().await?;
  let response = deny_app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/bodhi/v1/apps/mcps/mcp-uuid-2/mcp")
        .body(Body::empty())?
        .with_auth_context(ctx.clone()),
    )
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  let body: BodhiErrorResponse = response.json().await?;
  assert_eq!(
    Some("token_grant_error-mcp_forbidden".to_string()),
    body.error.code
  );

  // Granted instance clears the gate; absent upstream (get → None) surfaces as 404, never 403.
  let grant_app = proxy_router().await?;
  let response = grant_app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/bodhi/v1/apps/mcps/mcp-uuid-1/mcp")
        .body(Body::empty())?
        .with_auth_context(ctx),
    )
    .await?;
  assert_ne!(StatusCode::FORBIDDEN, response.status());
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

// ============================================================================
// POST /mcps - Create MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_success(test_mcp_entity: McpWithServerEntity) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let instance = test_mcp_entity.clone();

  mock
    .expect_create()
    .withf(|_, user_id, form| {
      user_id == "user123"
        && form.name == "DeepWiki MCP"
        && form.slug == "deepwiki"
        && form.mcp_server_id.as_deref() == Some("server-uuid-1")
        && form.auth_type == McpAuthType::Public
        && form.auth_config_id.is_none()
        && form.credentials.is_none()
        && form.oauth_token_id.is_none()
    })
    .times(1)
    .returning(move |_, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: Some("server-uuid-1".to_string()),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let body: Mcp = response.json().await?;
  assert_eq!("mcp-uuid-1", body.id);
  assert_eq!("deepwiki", body.slug);
  assert_eq!(McpAuthType::Public, body.auth_type);
  assert_eq!(None, body.auth_config_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_auth_config_id(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut instance = test_mcp_entity.clone();
  instance.auth_type = McpAuthType::Header;
  instance.auth_config_id = Some("auth-uuid-1".to_string());

  mock
    .expect_create()
    .withf(|_, _, form| {
      form.auth_type == McpAuthType::Header
        && form.auth_config_id.as_deref() == Some("auth-uuid-1")
        && form.credentials.is_none()
        && form.oauth_token_id.is_none()
    })
    .times(1)
    .returning(move |_, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: Some("server-uuid-1".to_string()),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Header,
    auth_config_id: Some("auth-uuid-1".to_string()),
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: Mcp = response.json().await?;
  assert_eq!(McpAuthType::Header, body.auth_type);
  assert_eq!(Some("auth-uuid-1".to_string()), body.auth_config_id);
  Ok(())
}

// ============================================================================
// PUT /mcps/{id} - Update MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_success(test_mcp_entity: McpWithServerEntity) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_entity.clone();
  updated.name = "Updated Name".to_string();

  mock
    .expect_update()
    .withf(|_, user_id, id, form| {
      user_id == "user123"
        && id == "mcp-uuid-1"
        && form.name == "Updated Name"
        && form.slug == "deepwiki"
        && form.auth_type == McpAuthType::Public
        && form.auth_config_id.is_none()
        && form.credentials.is_none()
        && form.oauth_token_id.is_none()
    })
    .times(1)
    .returning(move |_, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "Updated Name".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: None,
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/mcp-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: Mcp = response.json().await?;
  assert_eq!("Updated Name", body.name);
  Ok(())
}

// ============================================================================
// POST /mcps - Create with oauth_token_id
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_oauth_token_id(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut instance = test_mcp_entity.clone();
  instance.auth_type = McpAuthType::Oauth;
  instance.auth_config_id = Some("oauth-config-1".to_string());

  mock
    .expect_create()
    .withf(|_, _, form| {
      form.auth_type == McpAuthType::Oauth
        && form.auth_config_id.as_deref() == Some("oauth-config-1")
        && form.oauth_token_id.as_deref() == Some("token-123")
        && form.credentials.is_none()
    })
    .times(1)
    .returning(move |_, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: Some("server-uuid-1".to_string()),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Oauth,
    auth_config_id: Some("oauth-config-1".to_string()),
    credentials: None,
    oauth_token_id: Some("token-123".to_string()),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: Mcp = response.json().await?;
  assert_eq!(McpAuthType::Oauth, body.auth_type);
  assert_eq!(Some("oauth-config-1".to_string()), body.auth_config_id);
  Ok(())
}

// ============================================================================
// POST /mcps - Create with credentials
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_credentials(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut instance = test_mcp_entity.clone();
  instance.auth_type = McpAuthType::Header;
  instance.auth_config_id = Some("auth-uuid-1".to_string());

  mock
    .expect_create()
    .withf(|_, _, form| {
      form.auth_type == McpAuthType::Header
        && form.auth_config_id.as_deref() == Some("auth-uuid-1")
        && form.oauth_token_id.is_none()
        && form.credentials.as_ref().map(|c| {
          c.len() == 1
            && c[0].param_type == McpAuthParamType::Header
            && c[0].param_key == "Authorization"
            && c[0].value == "Bearer my-secret"
        }) == Some(true)
    })
    .times(1)
    .returning(move |_, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: Some("server-uuid-1".to_string()),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Header,
    auth_config_id: Some("auth-uuid-1".to_string()),
    credentials: Some(vec![McpAuthParamInput {
      param_type: McpAuthParamType::Header,
      param_key: "Authorization".to_string(),
      value: "Bearer my-secret".to_string(),
    }]),
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());
  Ok(())
}

// ============================================================================
// PUT /mcps/{id} - Update with credentials
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_change_credentials(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_entity.clone();
  updated.auth_type = McpAuthType::Header;
  updated.auth_config_id = Some("auth-uuid-1".to_string());

  mock
    .expect_update()
    .withf(|_, user_id, id, form| {
      user_id == "user123"
        && id == "mcp-uuid-1"
        && form.auth_type == McpAuthType::Header
        && form.auth_config_id.as_deref() == Some("auth-uuid-1")
        && form.oauth_token_id.is_none()
        && form.credentials.as_ref().map(|c| {
          c.len() == 2
            && c[0].param_type == McpAuthParamType::Header
            && c[0].param_key == "X-Api-Key"
            && c[0].value == "key-123"
            && c[1].param_type == McpAuthParamType::Query
            && c[1].param_key == "token"
            && c[1].value == "query-val"
        }) == Some(true)
    })
    .times(1)
    .returning(move |_, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: None,
    description: None,
    enabled: true,
    auth_type: McpAuthType::Header,
    auth_config_id: Some("auth-uuid-1".to_string()),
    credentials: Some(vec![
      McpAuthParamInput {
        param_type: McpAuthParamType::Header,
        param_key: "X-Api-Key".to_string(),
        value: "key-123".to_string(),
      },
      McpAuthParamInput {
        param_type: McpAuthParamType::Query,
        param_key: "token".to_string(),
        value: "query-val".to_string(),
      },
    ]),
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/mcp-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// PUT /mcps/{id} - Update with oauth_token_id
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_change_oauth_token(
  test_mcp_entity: McpWithServerEntity,
) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_entity.clone();
  updated.auth_type = McpAuthType::Oauth;
  updated.auth_config_id = Some("oauth-config-1".to_string());

  mock
    .expect_update()
    .withf(|_, user_id, id, form| {
      user_id == "user123"
        && id == "mcp-uuid-1"
        && form.auth_type == McpAuthType::Oauth
        && form.auth_config_id.as_deref() == Some("oauth-config-1")
        && form.oauth_token_id.as_deref() == Some("new-token-456")
        && form.credentials.is_none()
    })
    .times(1)
    .returning(move |_, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: None,
    description: None,
    enabled: true,
    auth_type: McpAuthType::Oauth,
    auth_config_id: Some("oauth-config-1".to_string()),
    credentials: None,
    oauth_token_id: Some("new-token-456".to_string()),
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/mcp-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// PUT /mcps/{id} - Update to clear auth (switch to public)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_clear_auth(test_mcp_entity: McpWithServerEntity) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let updated = test_mcp_entity.clone();
  // Entity already has auth_type: Public, auth_config_id: None

  mock
    .expect_update()
    .withf(|_, user_id, id, form| {
      user_id == "user123"
        && id == "mcp-uuid-1"
        && form.auth_type == McpAuthType::Public
        && form.auth_config_id.is_none()
        && form.credentials.is_none()
        && form.oauth_token_id.is_none()
    })
    .times(1)
    .returning(move |_, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: None,
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/mcp-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: Mcp = response.json().await?;
  assert_eq!(McpAuthType::Public, body.auth_type);
  assert_eq!(None, body.auth_config_id);
  Ok(())
}

// ============================================================================
// DELETE /mcps/{id}
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_mcp_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_delete()
    .withf(|_, user_id, id| user_id == "user123" && id == "mcp-uuid-1")
    .times(1)
    .returning(|_, _, _| Ok(()));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("DELETE")
    .uri("/mcps/mcp-uuid-1")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NO_CONTENT, response.status());
  Ok(())
}

// ============================================================================
// Error path tests for MCP create/update (Finding 14)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_service_error_returns_correct_status() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_create()
    .returning(|_, _, _| Err(McpError::McpServerNotFound("bad-server".into())));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "Test MCP".to_string(),
    slug: "test-mcp".to_string(),
    mcp_server_id: Some("bad-server".to_string()),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body: BodhiErrorResponse = response.json().await?;
  assert_eq!(
    Some("mcp_error-mcp_server_not_found".to_string()),
    body.error.code
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_slug_conflict_returns_409() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_create()
    .returning(|_, _, _| Err(McpError::SlugExists("test-mcp".into())));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "Test MCP".to_string(),
    slug: "test-mcp".to_string(),
    mcp_server_id: Some("server-uuid-1".to_string()),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CONFLICT, response.status());
  let body: BodhiErrorResponse = response.json().await?;
  assert_eq!(Some("mcp_error-slug_exists".to_string()), body.error.code);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_not_found_returns_404() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_update()
    .returning(|_, _, _, _| Err(McpError::McpNotFound("nonexistent-id".into())));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpRequest {
    name: "Updated MCP".to_string(),
    slug: "updated-mcp".to_string(),
    mcp_server_id: None,
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/nonexistent-id")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body: BodhiErrorResponse = response.json().await?;
  assert_eq!(Some("mcp_error-mcp_not_found".to_string()), body.error.code);
  Ok(())
}

// ============================================================================
// Integration tests (real DB)
// ============================================================================

use crate::test_utils::{
  build_test_router, create_authenticated_session, create_header_auth_config_in_db,
  session_request, session_request_with_body, setup_mcp_server_in_db,
};
use serde_json::{json, Value};

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_create_mcp_with_header_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  let auth_id =
    create_header_auth_config_in_db(&router, &user_cookie, &server_id, "Authorization").await?;

  let body = json!({
    "name": "Tavily Auth",
    "slug": "tavily-auth",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "header",
    "auth_config_id": auth_id
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let mcp: Value = response.json().await?;
  assert_eq!("header", mcp["auth_type"]);
  assert_eq!(auth_id, mcp["auth_config_id"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_create_mcp_with_public_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "name": "Public MCP",
    "slug": "public-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "public"
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let mcp: Value = response.json().await?;
  assert_eq!("public", mcp["auth_type"]);
  assert_eq!(Value::Null, mcp["auth_config_id"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_switch_auth_type() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let create_body = json!({
    "name": "My MCP",
    "slug": "my-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "public"
  });
  let create_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&create_body)?),
    ))
    .await?;
  let created: Value = create_resp.json().await?;
  let mcp_id = created["id"].as_str().unwrap();

  let auth_id =
    create_header_auth_config_in_db(&router, &user_cookie, &server_id, "X-Api-Key").await?;

  let update_body = json!({
    "name": "My MCP",
    "slug": "my-mcp",
    "enabled": true,
    "auth_type": "header",
    "auth_config_id": auth_id
  });
  let update_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "PUT",
      &format!("/bodhi/v1/mcps/{}", mcp_id),
      &user_cookie,
      Body::from(serde_json::to_string(&update_body)?),
    ))
    .await?;
  assert_eq!(StatusCode::OK, update_resp.status());

  let updated: Value = update_resp.json().await?;
  assert_eq!("header", updated["auth_type"]);
  assert_eq!(auth_id, updated["auth_config_id"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_keep_existing_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  let auth_id =
    create_header_auth_config_in_db(&router, &user_cookie, &server_id, "Authorization").await?;

  let create_body = json!({
    "name": "Keep Auth MCP",
    "slug": "keep-auth-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "header",
    "auth_config_id": auth_id
  });
  let create_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&create_body)?),
    ))
    .await?;
  let created: Value = create_resp.json().await?;
  let mcp_id = created["id"].as_str().unwrap();

  let update_body = json!({
    "name": "Renamed MCP",
    "slug": "keep-auth-mcp",
    "enabled": true,
    "auth_type": "header"
  });
  let update_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "PUT",
      &format!("/bodhi/v1/mcps/{}", mcp_id),
      &user_cookie,
      Body::from(serde_json::to_string(&update_body)?),
    ))
    .await?;
  assert_eq!(StatusCode::OK, update_resp.status());

  let updated: Value = update_resp.json().await?;
  assert_eq!("Renamed MCP", updated["name"]);
  assert_eq!("header", updated["auth_type"]);
  assert_eq!(auth_id, updated["auth_config_id"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_list_mcps_shows_auth_info() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  let auth_id =
    create_header_auth_config_in_db(&router, &user_cookie, &server_id, "X-Api-Key").await?;

  let public_body = json!({
    "name": "Public One",
    "slug": "public-one",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "public"
  });
  router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&public_body)?),
    ))
    .await?;

  let header_body = json!({
    "name": "Header One",
    "slug": "header-one",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "header",
    "auth_config_id": auth_id
  });
  router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&header_body)?),
    ))
    .await?;

  let list_resp = router
    .clone()
    .oneshot(session_request("GET", "/bodhi/v1/mcps", &user_cookie))
    .await?;
  assert_eq!(StatusCode::OK, list_resp.status());

  let list: Value = list_resp.json().await?;
  let mcps = list["mcps"].as_array().unwrap();
  assert_eq!(2, mcps.len());

  let public_mcp = mcps.iter().find(|m| m["slug"] == "public-one").unwrap();
  assert_eq!("public", public_mcp["auth_type"]);

  let header_mcp = mcps.iter().find(|m| m["slug"] == "header-one").unwrap();
  assert_eq!("header", header_mcp["auth_type"]);
  assert_eq!(auth_id, header_mcp["auth_config_id"].as_str().unwrap());
  Ok(())
}
