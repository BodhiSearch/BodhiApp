use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Manager-tier user management endpoints: unauthenticated access rejected (401) ---

#[rstest]
#[case::list_users("GET", "/bodhi/v1/users")]
#[case::change_role("PUT", "/bodhi/v1/users/some_user_id/role")]
#[case::remove_user("DELETE", "/bodhi/v1/users/some_user_id")]
#[tokio::test]
async fn test_user_management_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Manager-tier user management endpoints: insufficient role rejected (403) ---

#[rstest]
#[case::user("resource_user")]
#[case::power_user("resource_power_user")]
#[tokio::test]
async fn test_user_management_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints: Vec<(&str, &str)> = vec![
    ("GET", "/bodhi/v1/users"),
    ("PUT", "/bodhi/v1/users/some_user_id/role"),
    ("DELETE", "/bodhi/v1/users/some_user_id"),
  ];

  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role])
      .await
      .unwrap();

  for (method, path) in endpoints {
    let response = router
      .clone()
      .oneshot(session_request(method, path, &cookie))
      .await
      .unwrap();
    assert_eq!(
      StatusCode::FORBIDDEN,
      response.status(),
      "{role} should be forbidden from {method} {path}"
    );
  }
}

// --- Manager-tier user management endpoints: manager allowed ---
// All 3 endpoints (list_users, change_user_role, remove_user) call auth_service
// (MockAuthService with no expectations), which would panic. Since these endpoints
// share the same manager_session_apis route_layer (ResourceRole::Manager) as the
// access request endpoints tested in routes_users_access_request_auth_test.rs,
// the manager-allowed auth behavior is already proven there.
