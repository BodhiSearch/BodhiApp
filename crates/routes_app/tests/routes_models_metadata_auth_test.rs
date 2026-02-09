use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Metadata/queue endpoints (PowerUser session-only): unauthenticated access rejected (401) ---

#[rstest]
#[case::refresh_metadata("POST", "/bodhi/v1/models/refresh")]
#[case::queue_status("GET", "/bodhi/v1/queue")]
#[tokio::test]
async fn test_metadata_endpoints_reject_unauthenticated(
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

// --- Metadata/queue endpoints reject user role (403) ---

#[rstest]
#[case::refresh_metadata("POST", "/bodhi/v1/models/refresh")]
#[case::queue_status("GET", "/bodhi/v1/queue")]
#[tokio::test]
async fn test_metadata_endpoints_reject_user_role(#[case] method: &str, #[case] path: &str) {
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

// --- Metadata/queue endpoints allow power_user (not 401/403) ---
// Both endpoints in this tier call MockQueueProducer (which panics with no expectations):
// - refresh_metadata_handler calls queue_producer().enqueue() for bulk mode
// - queue_status_handler calls queue_producer().queue_status()
// These endpoints share the power_user_session_apis route_layer with token endpoints,
// which are tested in routes_api_token_auth_test.rs. The 401 and 403 tests above
// sufficiently prove the auth middleware is configured for PowerUser session-only access.
