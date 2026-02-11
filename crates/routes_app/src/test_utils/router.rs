use crate::build_routes;
use axum::{body::Body, http::Request, Router};
use server_core::{DefaultSharedContext, MockSharedContext, SharedContext};
use services::{
  test_utils::{access_token_claims, build_token, AppServiceStubBuilder, StubQueue, TEST_CLIENT_ID},
  AppService, SessionService, StubNetworkService,
};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

/// Builds a fully-composed test router with all services wired using real
/// in-memory implementations (SQLite, file-based data service, etc.).
///
/// Returns:
/// - `Router` - the fully composed router from `build_routes()` with session layer, auth middleware, etc.
/// - `Arc<dyn AppService>` - the app service handle for test data setup (e.g., db_service, data_service)
/// - `Arc<TempDir>` - temp directory ownership to keep it alive for the test duration
pub async fn build_test_router() -> anyhow::Result<(Router, Arc<dyn AppService>, Arc<TempDir>)> {
  let mut builder = AppServiceStubBuilder::default();
  let stub_queue: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);
  let stub_network: Arc<dyn services::NetworkService> =
    Arc::new(StubNetworkService { ip: Some("192.168.1.100".to_string()) });
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
  let app_service_stub = builder.build()?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set by builder");
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);
  let ctx: Arc<dyn SharedContext> = Arc::new(MockSharedContext::default());
  let router = build_routes(ctx, app_service.clone(), None);
  Ok((router, app_service, temp_home))
}

/// Creates an authenticated session in the session store with the specified roles.
///
/// This function:
/// 1. Builds a JWT with the specified roles in `resource_access` for the test client
/// 2. Creates a session `Record` with `access_token` set to the JWT
/// 3. Saves the record to the `AppSessionStore`
/// 4. Returns a cookie string suitable for use in request headers
///
/// # Arguments
/// * `session_service` - The session service providing access to the session store
/// * `roles` - Slice of role strings (e.g., `&["resource_user"]`, `&["resource_admin", "resource_user"]`)
///
/// # Returns
/// A cookie string like `bodhiapp_session_id=<session_id>` that can be used in request headers.
pub async fn create_authenticated_session(
  session_service: &dyn SessionService,
  roles: &[&str],
) -> anyhow::Result<String> {
  // Build JWT claims with specified roles
  let mut claims = access_token_claims();
  claims["resource_access"][TEST_CLIENT_ID]["roles"] = serde_json::json!(roles);

  // Build the signed JWT token
  let (token, _public_key) = build_token(claims)?;

  // Create a session record with the access_token
  let session_id = Id::default();
  let mut data = HashMap::new();
  data.insert(
    "access_token".to_string(),
    serde_json::Value::String(token),
  );

  let record = Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
  };

  // Save to the session store
  let store = session_service.get_session_store();
  store.save(&record).await?;

  // Return the cookie string matching the session cookie name used by SqliteSessionService
  Ok(format!("bodhiapp_session_id={}", session_id))
}

/// Builds an HTTP request with session authentication and same-origin headers.
///
/// Sets:
/// - `Cookie` header with the session cookie
/// - `Sec-Fetch-Site: same-origin` header (required for session auth path)
/// - `Host: localhost:1135` header (required for same-origin check)
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

/// Builds an HTTP request without any authentication headers.
///
/// Sets only the `Host` header for proper routing but includes no session cookie
/// or bearer token, simulating an unauthenticated request.
pub fn unauth_request(method: &str, path: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}

/// Builds an HTTP request with session authentication and a JSON body.
///
/// Sets:
/// - `Cookie` header with the session cookie
/// - `Sec-Fetch-Site: same-origin` header (required for session auth path)
/// - `Host: localhost:1135` header (required for same-origin check)
/// - `Content-Type: application/json` header
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

/// Builds an HTTP request without authentication but with a JSON body.
///
/// Sets:
/// - `Host` header for proper routing
/// - `Content-Type: application/json` header
pub fn unauth_request_with_body(method: &str, path: &str, body: Body) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Host", "localhost:1135")
    .header("Content-Type", "application/json")
    .body(body)
    .unwrap()
}

/// Builds a fully-composed router with live services for integration testing with real LLM inference.
///
/// This function creates a router that exercises the complete request flow through real services:
/// - Real HF cache discovery (discovers models from ~/.cache/huggingface/hub)
/// - Real llama.cpp binary execution (from crates/llama_server_proc/bin/)
/// - Real LocalDataService with live hub service
/// - DefaultSharedContext with DefaultServerFactory (spawns actual llama.cpp processes)
///
/// Returns:
/// - `Router` - fully composed router with session layer, auth middleware, DefaultSharedContext
/// - `Arc<dyn AppService>` - app service handle for test setup
/// - `Arc<dyn SharedContext>` - shared context for cleanup (call ctx.stop().await after test)
/// - `Arc<TempDir>` - temp directory ownership to keep it alive for test duration
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
  let stub_network: Arc<dyn services::NetworkService> =
    Arc::new(StubNetworkService { ip: Some("192.168.1.100".to_string()) });
  builder
    .with_live_services() // real HF cache + real binary path
    .with_data_service()
    .await // LocalDataService using live hub (discovers Qwen3)
    .with_db_service()
    .await
    .with_session_service()
    .await
    .with_secret_service()
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let app_service_stub = builder.build()?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set");

  let hub_service = app_service_stub.hub_service.clone().unwrap();
  let setting_service = app_service_stub.setting_service.clone().unwrap();
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);

  // DefaultSharedContext::new uses DefaultServerFactory (spawns real llama.cpp)
  let ctx: Arc<dyn SharedContext> =
    Arc::new(DefaultSharedContext::new(hub_service, setting_service));
  let router = build_routes(ctx.clone(), app_service.clone(), None);
  Ok((router, app_service, ctx, temp_home))
}
