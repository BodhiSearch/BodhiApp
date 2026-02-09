use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{build_test_router, unauth_request};
use rstest::rstest;
use tower::ServiceExt;

#[rstest]
#[case::ping("GET", "/ping")]
#[case::health("GET", "/health")]
#[tokio::test]
async fn test_public_system_endpoints_accessible_without_auth(
  #[case] method: &str,
  #[case] path: &str,
) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router.oneshot(unauth_request(method, path)).await.unwrap();
  assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn test_app_info_accessible_without_auth() {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request("GET", "/bodhi/v1/info"))
    .await
    .unwrap();
  let status = response.status();
  assert_ne!(status, StatusCode::UNAUTHORIZED);
  assert_ne!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_logout_accessible_without_auth() {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request("POST", "/bodhi/v1/logout"))
    .await
    .unwrap();
  let status = response.status();
  assert_ne!(status, StatusCode::UNAUTHORIZED);
  assert_ne!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_setup_accessible_without_auth() {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request("POST", "/bodhi/v1/setup"))
    .await
    .unwrap();
  let status = response.status();
  // Setup may return other errors (e.g., 422 for missing body, 400 for already setup)
  // but should never return auth errors
  assert_ne!(status, StatusCode::UNAUTHORIZED);
  assert_ne!(status, StatusCode::FORBIDDEN);
}
