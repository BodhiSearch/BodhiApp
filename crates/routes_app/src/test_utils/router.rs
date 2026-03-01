use crate::build_routes;
use axum::{body::Body, http::Request, Router};
use chrono::Utc;
use server_core::{DefaultSharedContext, MockSharedContext, SharedContext};
use services::{
  db::DbService,
  test_utils::{
    access_token_claims, build_token, AppServiceStubBuilder, StubNetworkService, StubQueue,
    TEST_CLIENT_ID,
  },
  AppInstance, AppService, SessionService, {ApiToken, TokenStatus},
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
    .with_app_instance(AppInstance::test_default())
    .await
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let app_service_stub = builder.build().await?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set by builder");
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);
  let ctx: Arc<dyn SharedContext> = Arc::new(MockSharedContext::default());
  let router = build_routes(ctx, app_service.clone(), None).await;
  Ok((router, app_service, temp_home))
}

/// Creates an authenticated session in the session store with the specified roles.
///
/// This function:
/// 1. Builds a JWT with the specified roles in `resource_access` for the test client
/// 2. Creates a session `Record` with `access_token` set to the JWT
/// 3. Saves the record to the `SessionStoreBackend`
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
  data.insert("access_token".to_string(), serde_json::Value::String(token));

  let record = Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
  };

  // Save to the session store
  let store = session_service.get_session_store();
  store.save(&record).await?;

  // Return the cookie string matching the session cookie name used by DefaultSessionService
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
    .with_app_instance(AppInstance::test_default())
    .await
    .queue_producer(stub_queue)
    .network_service(stub_network);
  let app_service_stub = builder.build().await?;
  let temp_home = app_service_stub
    .temp_home
    .clone()
    .expect("temp_home should be set");

  let hub_service = app_service_stub.hub_service.clone().unwrap();
  let setting_service = app_service_stub.setting_service.clone().unwrap();
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);

  // DefaultSharedContext::new uses DefaultServerFactory (spawns real llama.cpp)
  let ctx: Arc<dyn SharedContext> =
    Arc::new(DefaultSharedContext::new(hub_service, setting_service).await);
  let router = build_routes(ctx.clone(), app_service.clone(), None).await;
  Ok((router, app_service, ctx, temp_home))
}

/// Creates a test API token in the database and returns the raw token string.
///
/// The token follows the production format:
/// - Prefix: `bodhiapp_` + 8 random chars (used for DB lookup)
/// - Full token: `bodhiapp_` + deterministic test string
/// - Hash: SHA-256 of the full token stored in the database
///
/// # Arguments
/// * `db_service` - The database service to insert the token into
///
/// # Returns
/// The raw token string (e.g., `bodhiapp_testtoken_for_testing_purposes_only`) that can be
/// used as a `Bearer` token in request headers.
pub async fn create_test_api_token(db_service: &dyn DbService) -> anyhow::Result<String> {
  let token_str = format!("bodhiapp_testtoken_{}", Uuid::new_v4());
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let now = Utc::now();
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test API Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  db_service.create_api_token(&mut api_token).await?;

  Ok(token_str)
}

/// Builds an HTTP request with an API token in the `Authorization: Bearer` header.
///
/// Sets:
/// - `Authorization: Bearer {token}` header
/// - `Host: localhost:1135` header (required for routing)
pub fn api_token_request(method: &str, path: &str, token: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(path)
    .header("Authorization", format!("Bearer {}", token))
    .header("Host", "localhost:1135")
    .body(Body::empty())
    .unwrap()
}
