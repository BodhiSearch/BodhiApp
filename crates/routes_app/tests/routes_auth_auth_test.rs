use axum::http::StatusCode;
use routes_app::test_utils::{build_test_router, unauth_request};
use rstest::rstest;
use tower::ServiceExt;

// --- Optional-auth endpoints accept unauthenticated requests (NOT 401/403) ---

#[rstest]
#[case::auth_initiate("POST", "/bodhi/v1/auth/initiate")]
#[case::auth_callback("POST", "/bodhi/v1/auth/callback")]
#[case::apps_request_access("POST", "/bodhi/v1/apps/request-access")]
#[tokio::test]
async fn test_optional_auth_endpoints_accept_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED,
    status,
    "optional auth endpoint should not return 401 for {method} {path}"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "optional auth endpoint should not return 403 for {method} {path}"
  );
}
