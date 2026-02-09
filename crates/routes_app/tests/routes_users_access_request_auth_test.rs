use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Manager-tier access request endpoints: unauthenticated access rejected (401) ---

#[rstest]
#[case::list_pending("GET", "/bodhi/v1/access-requests/pending")]
#[case::list_all("GET", "/bodhi/v1/access-requests")]
#[case::approve("POST", "/bodhi/v1/access-requests/1/approve")]
#[case::reject("POST", "/bodhi/v1/access-requests/1/reject")]
#[tokio::test]
async fn test_access_request_endpoints_reject_unauthenticated(
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

// --- Manager-tier access request endpoints: insufficient role rejected (403) ---

#[rstest]
#[case::user("resource_user")]
#[case::power_user("resource_power_user")]
#[tokio::test]
async fn test_access_request_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints: Vec<(&str, &str)> = vec![
    ("GET", "/bodhi/v1/access-requests/pending"),
    ("GET", "/bodhi/v1/access-requests"),
    ("POST", "/bodhi/v1/access-requests/1/approve"),
    ("POST", "/bodhi/v1/access-requests/1/reject"),
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

// --- Manager-tier access request endpoints: manager allowed (not 401/403) ---
// Only list_pending and list_all are tested here because they call db_service (real TestDbService).
// approve and reject handlers call auth_service (MockAuthService with no expectations), which
// would panic. Since all 4 endpoints share the same manager_session_apis route_layer
// (ResourceRole::Manager), proving list_pending and list_all pass auth is sufficient.

#[rstest]
#[case::list_pending("GET", "/bodhi/v1/access-requests/pending")]
#[case::list_all("GET", "/bodhi/v1/access-requests")]
#[tokio::test]
async fn test_access_request_endpoints_allow_manager(
  #[case] method: &str,
  #[case] path: &str,
) {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_manager"],
  )
  .await
  .unwrap();
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await
    .unwrap();
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED,
    status,
    "manager should not get 401 for {method} {path}"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "manager should not get 403 for {method} {path}"
  );
}
