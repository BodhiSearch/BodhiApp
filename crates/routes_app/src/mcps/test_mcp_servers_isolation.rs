use crate::mcps::{ListMcpServersResponse, McpServerResponse};
use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::{mcp_servers_create, mcp_servers_index, mcp_servers_show, mcp_servers_update};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::{get, post, put},
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
  AppService, McpServerRequest, ResourceRole, Tenant, TenantRepository,
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
    .route(
      "/mcps/servers",
      get(mcp_servers_index).post(mcp_servers_create),
    )
    .route(
      "/mcps/servers/{id}",
      get(mcp_servers_show).put(mcp_servers_update),
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

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_server_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create MCP server in tenant A
  create_mcp_server(&router, &auth_a, "Server A", "https://mcp-a.example.com").await?;
  // Create MCP server in tenant B
  create_mcp_server(&router, &auth_b, "Server B", "https://mcp-b.example.com").await?;

  // List as tenant A -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps/servers")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpServersResponse>().await?;
  assert_eq!(1, list.mcp_servers.len());

  // List as tenant B -> only 1
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/mcps/servers")
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListMcpServersResponse>().await?;
  assert_eq!(1, list.mcp_servers.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_mcp_server_show_isolation(
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

  // Show that server as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/mcps/servers/{}", server_a.server.id))
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
async fn test_cross_tenant_mcp_server_update_isolation(
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

  // Update that server as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/mcps/servers/{}", server_a.server.id))
        .json(&McpServerRequest {
          url: "https://mcp-updated.example.com".to_string(),
          name: "Updated Server".to_string(),
          description: None,
          enabled: true,
          auth_config: None,
        })?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}
