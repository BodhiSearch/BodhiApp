use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::toolsets::toolsets_api_schemas::{ListToolsetsResponse, ToolsetResponse};
use crate::{toolsets_create, toolsets_destroy, toolsets_index, toolsets_show, toolsets_update};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::{delete, get, post, put},
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
  test_utils::{
    sea_context, AppServiceStubBuilder, SeaTestContext, TEST_TENANT_B_ID, TEST_TENANT_ID,
  },
  AppService, ResourceRole, Tenant, TenantRepository, ToolsetRequest,
};
use std::sync::Arc;
use tower::ServiceExt;

/// Returns (router, app_service, _ctx) — caller must hold `_ctx` to keep the SQLite temp dir alive.
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

  // Create both tenants with deterministic IDs via create_tenant_test
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_default())
    .await?;
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_tenant_b())
    .await?;

  // Enable builtin-exa-search toolset type for both tenants
  app_service
    .tool_service()
    .set_app_toolset_enabled(TEST_TENANT_ID, "builtin-exa-search", true, "admin")
    .await?;
  app_service
    .tool_service()
    .set_app_toolset_enabled(TEST_TENANT_B_ID, "builtin-exa-search", true, "admin")
    .await?;

  let router = Router::new()
    .route("/toolsets", get(toolsets_index).post(toolsets_create))
    .route(
      "/toolsets/{id}",
      get(toolsets_show)
        .put(toolsets_update)
        .delete(toolsets_destroy),
    )
    .with_state(app_service.clone());

  Ok((router, app_service, ctx))
}

fn make_toolset_request(slug: &str) -> ToolsetRequest {
  ToolsetRequest {
    toolset_type: Some("builtin-exa-search".to_string()),
    slug: slug.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    ..Default::default()
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_toolset_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create toolset as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-a"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create toolset as tenant B user A (same user, different tenant)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-b"))?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as tenant A -> only 1 toolset
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListToolsetsResponse>().await?;
  assert_eq!(1, list.toolsets.len());

  // List as tenant B -> only 1 toolset
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListToolsetsResponse>().await?;
  assert_eq!(1, list.toolsets.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_toolset_show_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create toolset as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-a"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<ToolsetResponse>().await?;
  let toolset_id = created.id;

  // Show that ID as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("/toolsets/{}", toolset_id))
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
async fn test_cross_tenant_toolset_update_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create toolset as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-a"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<ToolsetResponse>().await?;
  let toolset_id = created.id;

  // Update that ID as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/toolsets/{}", toolset_id))
        .json(&make_toolset_request("toolset-updated"))?
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
async fn test_cross_tenant_toolset_delete_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create toolset as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-a"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<ToolsetResponse>().await?;
  let toolset_id = created.id;

  // Delete that ID as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/toolsets/{}", toolset_id))
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
async fn test_intra_tenant_user_toolset_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_user_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);

  // Create toolset as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-ua"))?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create toolset as tenant A user B
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/toolsets")
        .json(&make_toolset_request("toolset-ub"))?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as user A -> only 1 toolset
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListToolsetsResponse>().await?;
  assert_eq!(1, list.toolsets.len());

  // List as user B -> only 1 toolset
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<ListToolsetsResponse>().await?;
  assert_eq!(1, list.toolsets.len());

  Ok(())
}
