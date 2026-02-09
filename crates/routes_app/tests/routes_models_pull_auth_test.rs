use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Pull/download endpoints (PowerUser-tier): unauthenticated access rejected (401) ---

#[rstest]
#[case::list_downloads("GET", "/bodhi/v1/modelfiles/pull")]
#[case::create_pull("POST", "/bodhi/v1/modelfiles/pull")]
#[case::get_download_status("GET", "/bodhi/v1/modelfiles/pull/some-id")]
#[tokio::test]
async fn test_pull_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Pull/download endpoints reject user role (403) ---

#[rstest]
#[case::list_downloads("GET", "/bodhi/v1/modelfiles/pull")]
#[case::create_pull("POST", "/bodhi/v1/modelfiles/pull")]
#[case::get_download_status("GET", "/bodhi/v1/modelfiles/pull/some-id")]
#[tokio::test]
async fn test_pull_endpoints_reject_user_role(#[case] method: &str, #[case] path: &str) {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_user"],
  )
  .await
  .unwrap();
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await
    .unwrap();
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "user should get 403 for {method} {path}"
  );
}

// --- Pull/download endpoints allow power_user (not 401/403) ---

#[rstest]
#[case::list_downloads("GET", "/bodhi/v1/modelfiles/pull")]
#[case::get_download_status("GET", "/bodhi/v1/modelfiles/pull/some-id")]
#[tokio::test]
async fn test_pull_endpoints_allow_power_user(#[case] method: &str, #[case] path: &str) {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_power_user"],
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
    "power_user should not get 401 for {method} {path}"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "power_user should not get 403 for {method} {path}"
  );
}
