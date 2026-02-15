//! POC test: Verify that ExternalTokenSimulator can bypass Keycloak token exchange
//! by seeding the cache, allowing toolset list endpoint access with a fake OAuth token.

mod utils;

use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use utils::ExternalTokenSimulator;

/// POC: External token with scope_user_user can access GET /bodhi/v1/toolsets
/// via cache bypass (no Keycloak needed).
///
/// This test:
/// 1. Builds a test router with real services (DefaultToolService, real DB)
/// 2. Uses ExternalTokenSimulator to create a fake OAuth token with scope_user_user
/// 3. Sends a request with the token as Bearer auth
/// 4. Asserts 200 OK (auth passed, endpoint accessible)
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_external_token_cache_bypass_toolsets_list() -> anyhow::Result<()> {
  use routes_app::build_routes;
  use server_core::MockSharedContext;
  use services::{
    test_utils::{AppServiceStubBuilder, StubQueue},
    DefaultExaService, DefaultToolService, StubNetworkService,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  // Build the router with real ToolService (backed by real DB)
  let mut builder = AppServiceStubBuilder::default();
  let stub_queue: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);
  let stub_network: Arc<dyn services::NetworkService> =
    Arc::new(StubNetworkService {
      ip: Some("192.168.1.100".to_string()),
    });
  builder
    .with_hub_service()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .with_session_service()
    .await
    .with_secret_service()
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let mut app_service_stub = builder.build()?;

  // Wire real DefaultToolService using the built stub's DB and time services
  let db_service = app_service_stub.db_service.clone().unwrap();
  let time_service = app_service_stub.time_service.clone().unwrap();
  let exa_service: Arc<dyn services::ExaService> = Arc::new(DefaultExaService::new());
  app_service_stub.tool_service =
    Some(Arc::new(DefaultToolService::new(db_service, exa_service, time_service)));

  let _temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set");
  let app_service: Arc<dyn services::AppService> = Arc::new(app_service_stub);
  let ctx: Arc<dyn server_core::SharedContext> = Arc::new(MockSharedContext::default());
  let router = build_routes(ctx, app_service.clone(), None);

  let simulator = ExternalTokenSimulator::new(&app_service);
  let bearer_token =
    simulator.create_token_with_scope("scope_user_user offline_access", "test-external-app")?;

  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/toolsets")
    .header("Authorization", format!("Bearer {}", bearer_token))
    .header("Host", "localhost:1135")
    .body(axum::body::Body::empty())?;

  let response = router.oneshot(request).await?;

  assert_eq!(
    StatusCode::OK,
    response.status(),
    "External OAuth token with scope_user_user should access toolsets list endpoint"
  );
  Ok(())
}

/// POC: External token without scope_user_user scope is rejected.
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_external_token_cache_bypass_missing_scope_rejected() -> anyhow::Result<()> {
  use routes_app::test_utils::build_test_router;
  use tower::ServiceExt;

  let (router, app_service, _temp) = build_test_router().await?;

  let simulator = ExternalTokenSimulator::new(&app_service);
  // Create token with only offline_access - no user scope
  let bearer_token =
    simulator.create_token_with_scope("offline_access", "test-external-app")?;

  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/toolsets")
    .header("Authorization", format!("Bearer {}", bearer_token))
    .header("Host", "localhost:1135")
    .body(axum::body::Body::empty())?;

  let response = router.oneshot(request).await?;

  // Should be rejected - missing scope_user_user
  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token without scope_user_user should be rejected"
  );
  Ok(())
}

/// POC: External token is rejected on session-only endpoints (GET /toolsets/{id}).
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_external_token_rejected_on_session_only_endpoint() -> anyhow::Result<()> {
  use routes_app::test_utils::build_test_router;
  use tower::ServiceExt;

  let (router, app_service, _temp) = build_test_router().await?;

  let simulator = ExternalTokenSimulator::new(&app_service);
  let bearer_token =
    simulator.create_token_with_scope("scope_user_user offline_access", "test-external-app")?;

  // GET /toolsets/{id} is session-only (no OAuth or API tokens)
  let request = axum::http::Request::builder()
    .method("GET")
    .uri("/bodhi/v1/toolsets/some-id")
    .header("Authorization", format!("Bearer {}", bearer_token))
    .header("Host", "localhost:1135")
    .body(axum::body::Body::empty())?;

  let response = router.oneshot(request).await?;

  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "External OAuth token should be rejected on session-only endpoint GET /toolsets/{{id}}"
  );
  Ok(())
}
