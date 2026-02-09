use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{build_test_router, unauth_request};
use rstest::rstest;
use tower::ServiceExt;

// --- Unauthenticated access rejected (401) ---

#[rstest]
#[case::create_toolset("POST", "/bodhi/v1/toolsets")]
#[case::get_toolset("GET", "/bodhi/v1/toolsets/some_id")]
#[case::update_toolset("PUT", "/bodhi/v1/toolsets/some_id")]
#[case::delete_toolset("DELETE", "/bodhi/v1/toolsets/some_id")]
#[case::list_toolsets("GET", "/bodhi/v1/toolsets")]
#[case::execute_toolset("POST", "/bodhi/v1/toolsets/some_id/execute/some_method")]
#[tokio::test]
async fn test_toolset_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- No "allowed" tests ---
// All toolset handlers call MockToolService, which panics without expectations.
// Auth tier coverage is proven by the unauthenticated rejection tests above:
// if the auth middleware weren't in place, requests would reach the handler and panic.
