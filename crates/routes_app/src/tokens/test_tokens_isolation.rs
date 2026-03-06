use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::tokens::tokens_api_schemas::{
  CreateTokenRequest, PaginatedTokenResponse, UpdateTokenRequest,
};
use crate::{tokens_create, tokens_index, tokens_update};
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
  AppService, ResourceRole, Tenant, TenantRepository, TokenScope, TokenStatus,
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

  let router = Router::new()
    .route("/api/tokens", get(tokens_index).post(tokens_create))
    .route("/api/tokens/{token_id}", put(tokens_update))
    .with_state(app_service.clone());

  Ok((router, app_service, ctx))
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_token_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create token as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Tenant A Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create token as tenant B user A (same user, different tenant)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Tenant B Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as tenant A -> only 1 token
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());

  // List as tenant B -> only 1 token
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_token_update_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create token as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Tenant A Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as tenant A to get the token ID
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());
  let token_id = &list.data[0].id;

  // Try to update that token as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tokens/{}", token_id))
        .json(&UpdateTokenRequest {
          name: "Updated".to_string(),
          status: TokenStatus::Active,
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
async fn test_intra_tenant_user_token_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_user_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);

  // Create token as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("User A Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create token as tenant A user B
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("User B Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as user A -> only 1 token
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());

  // List as user B -> only 1 token
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_token_update_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_user_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);

  // Create token as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("User A Token".to_string()),
          scope: TokenScope::User,
        })?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as user A to get the token ID
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(1, list.data.len());
  let token_id = &list.data[0].id;

  // Try to update as tenant A user B -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tokens/{}", token_id))
        .json(&UpdateTokenRequest {
          name: "Updated".to_string(),
          status: TokenStatus::Active,
        })?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}
