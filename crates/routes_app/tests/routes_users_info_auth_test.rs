use axum::http::StatusCode;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Optional-auth endpoints accept unauthenticated requests (NOT 401) ---

#[rstest]
#[case::user_info("GET", "/bodhi/v1/user")]
#[case::request_access("POST", "/bodhi/v1/user/request-access")]
#[case::request_status("GET", "/bodhi/v1/user/request-status")]
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
  // Optional auth endpoints should never return 401 - they accept unauthenticated requests
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

// --- Optional-auth endpoints allow authenticated users (not 401/403) ---

#[tokio::test]
async fn test_user_info_allows_authenticated() {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_user"],
  )
  .await
  .unwrap();
  let response = router
    .oneshot(session_request("GET", "/bodhi/v1/user", &cookie))
    .await
    .unwrap();
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED,
    status,
    "authenticated user should not get 401 for GET /bodhi/v1/user"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "authenticated user should not get 403 for GET /bodhi/v1/user"
  );
}
