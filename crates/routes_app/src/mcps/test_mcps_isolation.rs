use crate::mcps::{ListMcpsResponse, McpServerResponse};
use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::{mcp_servers_create, mcps_create, mcps_destroy, mcps_index, mcps_show, mcps_update};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::{get, post},
  Router,
};
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::AuthContext;
use services::{
  db::DbService,
  test_utils::{sea_context, AppServiceStubBuilder, SeaTestContext, TEST_TENANT_B_ID},
  AppService, Mcp, McpAuthType, McpRequest, McpServerRequest, ResourceRole, Tenant,
};
use std::sync::Arc;
use tower::ServiceExt;

/// Returns (router, app_service, _ctx) -- caller must hold `_ctx` to keep the SQLite temp dir alive.
async fn isolation_router(
  db_type: &str,
) -> anyhow::Result<(Router, Arc<dyn AppService>, SeaTestContext)> {
  let ctx = sea_context(db_type).await;
  let db_svc: Arc<dyn DbService> = Arc::new(ctx.service.clone());
  let mut builder = AppServiceStubBuilder::default();
  builder
    .db_service(db_svc.clone())
    .with_tenant_service()
    .await;
  let app_service: Arc<dyn AppService> = Arc::new(builder.build().await?);

  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_default())
    .await?;
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_tenant_b())
    .await?;

  let router = Router::new()
    .route("/mcps/servers", post(mcp_servers_create))
    .route("/mcps", get(mcps_index).post(mcps_create))
    .route(
      "/mcps/{id}",
      get(mcps_show).put(mcps_update).delete(mcps_destroy),
    )
    .with_state(app_service.clone());

  Ok((router, app_service, ctx))
}

async fn create_mcp_server(
  router: &Router,
  auth: &AuthContext,
  name: &str,
  url: &str,
) -> anyhow::Result<McpServerResponse> {
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/mcps/servers")
        .json(&McpServerRequest {
          url: url.to_string(),
          name: name.to_string(),
          description: None,
          enabled: true,
          auth_config: None,
        })?
        .with_auth_context(auth.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  Ok(response.json::<McpServerResponse>().await?)
}

async fn create_mcp_instance(
  router: &Router,
  auth: &AuthContext,
  name: &str,
  slug: &str,
  mcp_server_id: &str,
) -> anyhow::Result<Mcp> {
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/mcps")
        .json(&McpRequest {
          name: name.to_string(),
          slug: slug.to_string(),
          mcp_server_id: Some(mcp_server_id.to_string()),
          description: None,
          enabled: true,
          tools_cache: None,
          tools_filter: None,
          auth_type: McpAuthType::Public,
          auth_uuid: None,
        })?
        .with_auth_context(auth.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  Ok(response.json::<Mcp>().await?)
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create MCP server in tenant A
  let server_a =
    create_mcp_server(&router, &auth_a, "Server A", "https://mcp-a.example.com").await?;
  // Create MCP server in tenant B
  let server_b =
    create_mcp_server(&router, &auth_b, "Server B", "https://mcp-b.example.com").await?;

  // Create MCP instance in tenant A
  create_mcp_instance(&router, &auth_a, "MCP A", "mcp-a", &server_a.server.id).await?;
  // Create MCP instance in tenant B
  create_mcp_instance(&router, &auth_b, "MCP B", "mcp-b", &server_b.server.id).await?;

  // List as tenant A -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpsResponse>().await?;
  assert_eq!(1, list.mcps.len());

  // List as tenant B -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpsResponse>().await?;
  assert_eq!(1, list.mcps.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_show_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create MCP server and instance in tenant A
  let server_a =
    create_mcp_server(&router, &auth_a, "Server A", "https://mcp-a.example.com").await?;
  let mcp_a = create_mcp_instance(&router, &auth_a, "MCP A", "mcp-a", &server_a.server.id).await?;

  // Show that MCP as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/mcps/{}", mcp_a.id))
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_update_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create MCP server and instance in tenant A
  let server_a =
    create_mcp_server(&router, &auth_a, "Server A", "https://mcp-a.example.com").await?;
  let mcp_a = create_mcp_instance(&router, &auth_a, "MCP A", "mcp-a", &server_a.server.id).await?;

  // Update that MCP as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/mcps/{}", mcp_a.id))
        .json(&McpRequest {
          name: "Updated MCP".to_string(),
          slug: "mcp-a".to_string(),
          mcp_server_id: Some(server_a.server.id.clone()),
          description: None,
          enabled: true,
          tools_cache: None,
          tools_filter: None,
          auth_type: McpAuthType::Public,
          auth_uuid: None,
        })?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_delete_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create MCP server and instance in tenant A
  let server_a =
    create_mcp_server(&router, &auth_a, "Server A", "https://mcp-a.example.com").await?;
  let mcp_a = create_mcp_instance(&router, &auth_a, "MCP A", "mcp-a", &server_a.server.id).await?;

  // Delete that MCP as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/mcps/{}", mcp_a.id))
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_mcp_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_user_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);

  // Create MCP server in tenant A (either user can see it, servers are tenant-scoped)
  let server = create_mcp_server(
    &router,
    &auth_user_a,
    "Shared Server",
    "https://mcp-shared.example.com",
  )
  .await?;

  // Create MCP instance as user A in tenant A
  create_mcp_instance(
    &router,
    &auth_user_a,
    "User A MCP",
    "user-a-mcp",
    &server.server.id,
  )
  .await?;
  // Create MCP instance as user B in tenant A
  create_mcp_instance(
    &router,
    &auth_user_b,
    "User B MCP",
    "user-b-mcp",
    &server.server.id,
  )
  .await?;

  // List as user A -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpsResponse>().await?;
  assert_eq!(1, list.mcps.len());

  // List as user B -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps")
        .body(Body::empty())?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpsResponse>().await?;
  assert_eq!(1, list.mcps.len());

  Ok(())
}
