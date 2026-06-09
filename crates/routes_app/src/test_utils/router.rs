use crate::build_routes;
use axum::{body::Body, http::Request, Router};
use chrono::Utc;
use server_core::{DefaultSharedContext, LocalLlamaImpl, SharedContext};
use services::{
  db::DbService,
  inference::LocalLlama,
  test_utils::{
    access_token_claims, build_token, AppServiceStubBuilder, StubNetworkService, StubQueue,
    TEST_CLIENT_ID, TEST_TENANT_ID,
  },
  AppService, DefaultAiApiClientFactory, SessionService, Tenant, {TokenEntity, TokenStatus},
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};
use uuid::Uuid;

/// Fully-composed test router with services wired to real in-memory implementations
/// (SQLite, file-based data service, etc.). Returned `TempDir` must outlive the test.
pub async fn build_test_router() -> anyhow::Result<(Router, Arc<dyn AppService>, Arc<TempDir>)> {
  let mut builder = AppServiceStubBuilder::default();
  let stub_queue: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);
  let stub_network: Arc<dyn services::NetworkService> = Arc::new(StubNetworkService {
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
    .with_tenant(Tenant::test_default())
    .await
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let app_service_stub = builder.build().await?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set by builder");
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);
  let router = build_routes(app_service.clone(), None).await;
  Ok((router, app_service, temp_home))
}

/// `roles` go into `resource_access` for the test client; returns a
/// `bodhiapp_session_id=<id>` cookie string usable in request headers.
pub async fn create_authenticated_session(
  session_service: &dyn SessionService,
  roles: &[&str],
) -> anyhow::Result<String> {
  let mut claims = access_token_claims();
  claims["resource_access"][TEST_CLIENT_ID]["roles"] = serde_json::json!(roles);

  let (token, _public_key) = build_token(claims)?;

  // namespaced access_token + active_client_id
  let session_id = Id::default();
  let mut data = HashMap::new();
  let access_key = format!("{}:access_token", TEST_CLIENT_ID);
  data.insert(access_key, serde_json::Value::String(token));
  data.insert(
    "active_client_id".to_string(),
    serde_json::Value::String(TEST_CLIENT_ID.to_string()),
  );

  let record = Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
  };

  let store = session_service.get_session_store();
  store.save(&record).await?;

  // cookie name must match DefaultSessionService
  Ok(format!("bodhiapp_session_id={}", session_id))
}

/// `Sec-Fetch-Site: same-origin` is required for the session auth path; `Host` for the same-origin check.
pub fn session_request(method: &str, path: &str, session_cookie: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Cookie", session_cookie)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}

pub fn unauth_request(method: &str, path: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}

/// `Sec-Fetch-Site: same-origin` is required for the session auth path; `Host` for the same-origin check.
pub fn session_request_with_body(
  method: &str,
  path: &str,
  session_cookie: &str,
  body: Body,
) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Cookie", session_cookie)
    .header("Sec-Fetch-Site", "same-origin")
    .header("Host", "localhost:1135")
    .header("Content-Type", "application/json")
    .body(body)
    .unwrap()
}

pub fn unauth_request_with_body(method: &str, path: &str, body: Body) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Host", "localhost:1135")
    .header("Content-Type", "application/json")
    .body(body)
    .unwrap()
}

/// Exercises the full request flow through real services: HF cache discovery,
/// real llama.cpp binary execution, DefaultSharedContext spawning actual llama.cpp
/// processes. Returned `SharedContext` needs `ctx.stop().await` after the test.
///
/// # Prerequisites
/// - Pre-downloaded model at `~/.cache/huggingface/hub/` (e.g., ggml-org/Qwen3-1.7B-GGUF)
/// - llama.cpp binary at `crates/llama_server_proc/bin/{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}`
pub async fn build_live_test_router() -> anyhow::Result<(
  Router,
  Arc<dyn AppService>,
  Arc<dyn SharedContext>,
  Arc<TempDir>,
)> {
  let mut builder = AppServiceStubBuilder::default();
  let stub_queue: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);
  let stub_network: Arc<dyn services::NetworkService> = Arc::new(StubNetworkService {
    ip: Some("192.168.1.100".to_string()),
  });
  builder
    .with_live_services()
    .await
    .with_data_service()
    .await
    .with_db_service()
    .await
    .with_session_service()
    .await
    .with_tenant(Tenant::test_default())
    .await
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let mut app_service_stub = builder.build().await?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set");

  let hub_service = app_service_stub.hub_service.clone().unwrap();
  let setting_service = app_service_stub.setting_service.clone().unwrap();

  // DefaultSharedContext::new uses DefaultServerFactory (spawns real llama.cpp)
  let ctx: Arc<dyn SharedContext> =
    Arc::new(DefaultSharedContext::new(hub_service, setting_service).await);

  // Wire LocalLlamaImpl with the real SharedContext and rebuild the factory
  // with local_llama support so the unified AiApiClientFactory can dispatch
  // local + remote uniformly.
  let keep_alive_secs = app_service_stub
    .setting_service
    .as_ref()
    .expect("setting_service should be set")
    .keep_alive()
    .await;
  let local_llama: Arc<dyn LocalLlama> =
    Arc::new(LocalLlamaImpl::new(ctx.clone(), keep_alive_secs));
  app_service_stub.local_llama = Some(local_llama.clone());
  app_service_stub.ai_api_client_factory = Some(Arc::new(
    DefaultAiApiClientFactory::new()?.with_local_llama(local_llama),
  ));

  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);

  let router = build_routes(app_service.clone(), None).await;
  Ok((router, app_service, ctx, temp_home))
}

/// Token follows the production format: prefix `bodhiapp_` + 8 chars (DB lookup key),
/// SHA-256 of the full token stored in the DB. Returns the raw token for `Bearer` use.
pub async fn create_test_api_token(db_service: &dyn DbService) -> anyhow::Result<String> {
  let token_str = format!("bodhiapp_testtoken_{}", Uuid::new_v4());
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let now = Utc::now();
  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test API Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  Ok(token_str)
}

pub fn cors_preflight_request(path: &str, method: &str, origin: &str) -> Request<Body> {
  Request::builder()
    .method("OPTIONS")
    .uri(path)
    .header("Origin", origin)
    .header("Access-Control-Request-Method", method)
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}

pub fn request_with_origin(method: &str, path: &str, origin: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Origin", origin)
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}

pub fn api_token_request(method: &str, path: &str, token: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Authorization", format!("Bearer {}", token))
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}
