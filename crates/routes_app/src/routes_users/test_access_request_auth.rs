use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;

// Auth tier: Manager (session-only) - access request management requires resource_manager or resource_admin role

#[anyhow_trace]
#[rstest]
#[case::list_pending("GET", "/bodhi/v1/access-requests/pending")]
#[case::list_all("GET", "/bodhi/v1/access-requests")]
#[case::approve("POST", "/bodhi/v1/access-requests/1/approve")]
#[case::reject("POST", "/bodhi/v1/access-requests/1/reject")]
#[tokio::test]
async fn test_access_request_endpoints_reject_unauthenticated(
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
async fn test_access_request_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/access-requests/pending"),
    ("GET", "/bodhi/v1/access-requests"),
    ("POST", "/bodhi/v1/access-requests/1/approve"),
    ("POST", "/bodhi/v1/access-requests/1/reject")
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

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_access_request_endpoints_allow_manager_and_admin(
  #[values("resource_manager", "resource_admin")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/access-requests/pending"),
    ("GET", "/bodhi/v1/access-requests")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  // Both GET endpoints are safe: db_service returns empty list, no MockAuthService expectations
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// VIOLATION: POST endpoints for access request management cannot be added to allow test
// Reason: POST /access-requests/{id}/approve calls auth_service.assign_user_role() requiring MockAuthService
// These cannot work with build_test_router() without mock expectations.
