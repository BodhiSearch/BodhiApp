use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use rstest::rstest;
use tower::ServiceExt;

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_app_info_accessible_without_auth() -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router
    .oneshot(unauth_request("GET", "/bodhi/v1/info"))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_logout_accessible_without_auth() -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router
    .oneshot(unauth_request("POST", "/bodhi/v1/logout"))
    .await?;
  // Logout without session returns 200 (no-op) or 400 (session not found) - both are valid non-auth responses
  let status = response.status();
  assert!(
    status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
    "Expected 200 or 400, got {}",
    status
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setup_with_valid_body_no_auth_required() -> anyhow::Result<()> {
  use crate::test_utils::build_test_router;
  use axum::body::Body;
  use axum::http::Request;
  let (router, _app_service, _temp) = build_test_router().await?;

  // AppInstanceService from build_test_router() defaults to AppStatus::Ready
  let body = serde_json::json!({
    "name": "Test Server Name",
    "description": "Test description"
  });

  let req = Request::builder()
    .method("POST")
    .uri("/bodhi/v1/setup")
    .header("Content-Type", "application/json")
    .header("Host", "localhost:1135")
    .body(Body::from(serde_json::to_string(&body)?))
    .unwrap();

  let response = router.oneshot(req).await?;
  // Should not return auth errors (401/403) - MockAuthService will panic without expectation,
  // so we verify the route is not auth-protected by checking it doesn't return 401/403
  let status = response.status();
  assert!(
    status != StatusCode::UNAUTHORIZED && status != StatusCode::FORBIDDEN,
    "Expected non-auth error, got {}",
    status
  );
  Ok(())
}
