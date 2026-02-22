use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;

// Auth tier: Manager (session-only) - these endpoints require resource_manager or resource_admin role

#[anyhow_trace]
#[rstest]
#[case::list_users("GET", "/bodhi/v1/users")]
#[case::update_user_role("PUT", "/bodhi/v1/users/some_user_id/role")]
#[case::delete_user("DELETE", "/bodhi/v1/users/some_user_id")]
#[tokio::test]
async fn test_user_management_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_user_management_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/users"),
    ("PUT", "/bodhi/v1/users/some_user_id/role"),
    ("DELETE", "/bodhi/v1/users/some_user_id")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
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

// VIOLATION: Allow test for user management endpoints cannot be added
// Reason: All endpoints (GET /users, PUT /users/{id}/role, DELETE /users/{id})
// require MockAuthService expectations:
// - GET /users calls auth_service.list_users() requiring MockAuthService
// - PUT /users/{id}/role calls auth_service.assign_user_role() requiring MockAuthService
// - DELETE /users/{id} calls auth_service.remove_user() requiring MockAuthService
// These cannot work with build_test_router() without mock expectations.
