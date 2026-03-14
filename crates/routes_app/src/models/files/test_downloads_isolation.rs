use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::{models_pull_index, models_pull_show};
use anyhow_trace::anyhow_trace;
use axum::{body::Body, http::Request, routing::get, Router};
use chrono::Utc;
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use server_core::test_utils::ResponseTestExt;
use services::AuthContext;
use services::{
  db::DbService,
  test_utils::{sea_context, AppServiceStubBuilder, SeaTestContext, TEST_TENANT_B_ID},
  AppService, DownloadRequestEntity, PaginatedDownloadResponse, ResourceRole, Tenant,
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

  // Create both tenants with deterministic IDs
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_default())
    .await?;
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_tenant_b())
    .await?;

  let router = Router::new()
    .route("/bodhi/v1/models/files/pull", get(models_pull_index))
    .route("/bodhi/v1/models/files/pull/{id}", get(models_pull_show))
    .with_state(app_service.clone());

  Ok((router, app_service, ctx))
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_download_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  let now = Utc::now();

  // Seed download in tenant A
  let tenant_a_id = auth_a.require_tenant_id()?;
  let download_a =
    DownloadRequestEntity::new_pending(tenant_a_id, "org-a/model-a", "model-a.gguf", now);
  app_service
    .db_service()
    .create_download_request(&download_a)
    .await?;

  // Seed download in tenant B
  let tenant_b_id = auth_b.require_tenant_id()?;
  let download_b =
    DownloadRequestEntity::new_pending(tenant_b_id, "org-b/model-b", "model-b.gguf", now);
  app_service
    .db_service()
    .create_download_request(&download_b)
    .await?;

  // List as tenant A -> only 1 download
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/bodhi/v1/models/files/pull")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedDownloadResponse>().await?;
  assert_eq!(1, list.data.len());
  assert_eq!("org-a/model-a", list.data[0].repo);

  // List as tenant B -> only 1 download
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/bodhi/v1/models/files/pull")
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedDownloadResponse>().await?;
  assert_eq!(1, list.data.len());
  assert_eq!("org-b/model-b", list.data[0].repo);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_download_show_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  let now = Utc::now();

  // Seed download in tenant A
  let tenant_a_id = auth_a.require_tenant_id()?;
  let download_a =
    DownloadRequestEntity::new_pending(tenant_a_id, "org-a/model-a", "model-a.gguf", now);
  let download_a_id = download_a.id.clone();
  app_service
    .db_service()
    .create_download_request(&download_a)
    .await?;

  // Show download as tenant A -> 200
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/bodhi/v1/models/files/pull/{}", download_a_id))
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  // Show same download as tenant B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/bodhi/v1/models/files/pull/{}", download_a_id))
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}
