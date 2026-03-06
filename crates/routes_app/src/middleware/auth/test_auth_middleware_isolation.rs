use crate::middleware::auth::{auth_middleware, optional_auth_middleware};
use crate::test_utils::setup_env;
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  extract::Extension,
  http::{HeaderMap, Request, StatusCode},
  middleware::from_fn_with_state,
  response::{IntoResponse, Response},
  routing::get,
  Json, Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serial_test::serial;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  db::DbService,
  test_utils::{
    access_token_claims, build_token, sea_context, AppServiceStubBuilder, SeaTestContext,
    TEST_CLIENT_ID, TEST_TENANT_B_ID, TEST_TENANT_ID,
  },
  AppService, AuthContext, DefaultSessionService, DefaultTenantService, SessionService, Tenant,
  TenantRepository,
};
use std::sync::Arc;
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestResponse {
  client_id: Option<String>,
  tenant_id: Option<String>,
  user_id: Option<String>,
  is_authenticated: bool,
  path: String,
}

async fn test_handler(
  auth_context: Option<Extension<AuthContext>>,
  _headers: HeaderMap,
  path: &str,
) -> Response {
  let (client_id, tenant_id, user_id, is_authenticated) =
    if let Some(Extension(ctx)) = &auth_context {
      (
        ctx.client_id().map(|s| s.to_string()),
        ctx.tenant_id().map(|s| s.to_string()),
        ctx.user_id().map(|s| s.to_string()),
        ctx.is_authenticated(),
      )
    } else {
      (None, None, None, false)
    };
  (
    StatusCode::IM_A_TEAPOT,
    Json(TestResponse {
      client_id,
      tenant_id,
      user_id,
      is_authenticated,
      path: path.to_string(),
    }),
  )
    .into_response()
}

fn isolation_test_router(app_service: Arc<dyn AppService>) -> Router {
  Router::new()
    .merge(
      Router::new()
        .route(
          "/with_auth",
          get(
            |auth_context: Option<Extension<AuthContext>>, headers: HeaderMap| async move {
              test_handler(auth_context, headers, "/with_auth").await
            },
          ),
        )
        .route_layer(from_fn_with_state(app_service.clone(), auth_middleware)),
    )
    .merge(
      Router::new()
        .route(
          "/with_optional_auth",
          get(
            |auth_context: Option<Extension<AuthContext>>, headers: HeaderMap| async move {
              test_handler(auth_context, headers, "/with_optional_auth").await
            },
          ),
        )
        .route_layer(from_fn_with_state(
          app_service.clone(),
          optional_auth_middleware,
        )),
    )
    .layer(app_service.session_service().session_layer())
    .with_state(app_service)
}

async fn isolation_app_service(
  db_type: &str,
  mode: &str,
  session_service: Arc<dyn SessionService>,
) -> anyhow::Result<(Arc<dyn AppService>, SeaTestContext)> {
  let ctx = sea_context(db_type).await;
  let db_svc: Arc<dyn DbService> = Arc::new(ctx.service.clone());
  let mut builder = AppServiceStubBuilder::default();
  builder
    .db_service(db_svc.clone())
    .with_tenant_service()
    .await;
  builder.session_service(session_service);

  let app_service: Arc<dyn AppService> = Arc::new(builder.build().await?);

  // Always create tenant A
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_default())
    .await?;
  // Multi-tenant adds tenant B
  if mode == "multi-tenant" {
    app_service
      .db_service()
      .create_tenant_test(&Tenant::test_tenant_b())
      .await?;
  }

  Ok((app_service, ctx))
}

fn session_token_for_tenant(client_id: &str, roles: &[&str]) -> anyhow::Result<String> {
  let mut claims = access_token_claims();
  claims["azp"] = json!(client_id);
  claims["resource_access"] = json!({ client_id: { "roles": roles } });
  let (token, _) = build_token(claims)?;
  Ok(token)
}

// =============================================================================
// Test Group 1: Session token tenant resolution
// =============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_session_resolves_tenant_a_from_azp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
  #[values("standalone", "multi-tenant")] mode: &str,
) -> anyhow::Result<()> {
  let tmp = TempDir::new()?;
  let dbfile = tmp.path().join("session.db");
  let session_service: Arc<dyn SessionService> =
    Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let (app_service, _ctx) = isolation_app_service(db_type, mode, session_service.clone()).await?;

  let token = session_token_for_tenant(TEST_CLIENT_ID, &["resource_admin"])?;
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.clone()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let router = isolation_test_router(app_service);
  let req = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let body = response.json::<TestResponse>().await?;
  assert_eq!(Some(TEST_CLIENT_ID.to_string()), body.client_id);
  assert_eq!(Some(TEST_TENANT_ID.to_string()), body.tenant_id);
  assert_eq!(true, body.is_authenticated);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_session_resolves_tenant_b_from_azp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let tmp = TempDir::new()?;
  let dbfile = tmp.path().join("session.db");
  let session_service: Arc<dyn SessionService> =
    Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let (app_service, _ctx) =
    isolation_app_service(db_type, "multi-tenant", session_service.clone()).await?;

  let token = session_token_for_tenant("test-client-b", &["resource_admin"])?;
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.clone()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let router = isolation_test_router(app_service);
  let req = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let body = response.json::<TestResponse>().await?;
  assert_eq!(Some("test-client-b".to_string()), body.client_id);
  assert_eq!(Some(TEST_TENANT_B_ID.to_string()), body.tenant_id);
  assert_eq!(true, body.is_authenticated);
  Ok(())
}

// =============================================================================
// Test Group 3: Cross-tenant rejection / fallback
// =============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_session_rejects_unknown_azp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
  #[values("standalone", "multi-tenant")] mode: &str,
) -> anyhow::Result<()> {
  let tmp = TempDir::new()?;
  let dbfile = tmp.path().join("session.db");
  let session_service: Arc<dyn SessionService> =
    Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let (app_service, _ctx) = isolation_app_service(db_type, mode, session_service.clone()).await?;

  let token = session_token_for_tenant("nonexistent-client", &["resource_admin"])?;
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.clone()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let router = isolation_test_router(app_service);

  // Strict middleware: should return 500 (TenantError::NotFound is InternalServer)
  let req = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let response = router.clone().oneshot(req).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  let body: Value = response.json().await?;
  assert_eq!("tenant_error-not_found", body["error"]["code"]);

  // Optional middleware: should fall back to Anonymous
  let req = Request::get("/with_optional_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let body = response.json::<TestResponse>().await?;
  assert_eq!(false, body.is_authenticated);
  assert_eq!(None, body.client_id);
  assert_eq!(None, body.tenant_id);
  Ok(())
}
