use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use services::AppService;
use tower::ServiceExt;

// Test that toolset endpoints reject unauthenticated requests
// All toolset endpoints require User-level session authentication
// No allow test needed - MockToolService panics without expectations, proving auth layer works
#[anyhow_trace]
#[rstest]
#[case::create_toolset("POST", "/bodhi/v1/toolsets")]
#[case::list_toolsets("GET", "/bodhi/v1/toolsets")]
#[case::get_toolset("GET", "/bodhi/v1/toolsets/some_id")]
#[case::update_toolset("PUT", "/bodhi/v1/toolsets/some_id")]
#[case::delete_toolset("DELETE", "/bodhi/v1/toolsets/some_id")]
#[case::execute_toolset("POST", "/bodhi/v1/toolsets/some_id/tools/some_method/execute")]
#[case::list_toolset_types("GET", "/bodhi/v1/toolset_types")]
#[case::enable_toolset_type("PUT", "/bodhi/v1/toolset_types/some_type/app-config")]
#[case::disable_toolset_type("DELETE", "/bodhi/v1/toolset_types/some_type/app-config")]
#[tokio::test]
async fn test_toolset_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

// Test that toolset_type endpoints (Admin-only) reject insufficient roles
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_toolset_type_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user", "resource_manager")] role: &str,
  #[values(
    ("PUT", "/bodhi/v1/toolset_types/some_type/app-config"),
    ("DELETE", "/bodhi/v1/toolset_types/some_type/app-config")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

// Test that any resource_* role can access read-only toolset_type endpoints
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_toolset_type_read_endpoints_allow_any_resource_role(
  #[values(
    "resource_user",
    "resource_power_user",
    "resource_manager",
    "resource_admin"
  )]
  role: &str,
  #[values(
    ("GET", "/bodhi/v1/toolset_types")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(
    StatusCode::OK,
    response.status(),
    "{role} should be allowed to access {method} {path}"
  );
  Ok(())
}

// No allow test for toolset_type endpoints - they use MockToolService which panics without expectations
// The 401 and 403 tests verify the auth layer works correctly

// VIOLATION: No allow test for toolset CRUD endpoints (including list)
// All toolset endpoints (create, list, get, update, delete, execute) use MockToolService
// which panics without expectations. Cannot migrate to build_test_router() without refactoring
// the handlers to use real services or injecting mock expectations via AppServiceStubBuilder.
// The 401 test proves the auth layer works correctly.

// Test that API tokens (bodhiapp_*) are rejected on session-only toolset endpoints.
// These endpoints only accept session auth (and some accept OAuth), but never API tokens.
#[anyhow_trace]
#[rstest]
#[case::list("GET", "/bodhi/v1/toolsets")]
#[case::get("GET", "/bodhi/v1/toolsets/some-id")]
#[case::update("PUT", "/bodhi/v1/toolsets/some-id")]
#[case::delete("DELETE", "/bodhi/v1/toolsets/some-id")]
#[case::execute("POST", "/bodhi/v1/toolsets/some-id/tools/some-method/execute")]
#[tokio::test]
async fn test_toolset_endpoints_reject_api_token(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{api_token_request, build_test_router, create_test_api_token};
  let (router, app_service, _temp) = build_test_router().await?;
  let token = create_test_api_token(app_service.db_service().as_ref()).await?;
  let response = router
    .oneshot(api_token_request(method, path, &token))
    .await?;
  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "API token should be rejected on session-only endpoint {method} {path}"
  );
  Ok(())
}
