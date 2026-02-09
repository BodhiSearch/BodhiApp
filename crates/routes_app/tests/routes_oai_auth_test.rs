use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- OAI endpoints (User-tier): unauthenticated access rejected (401) ---

#[rstest]
#[case::list_models("GET", "/v1/models")]
#[case::get_model("GET", "/v1/models/some_model")]
#[case::chat_completions("POST", "/v1/chat/completions")]
#[case::embeddings("POST", "/v1/embeddings")]
#[tokio::test]
async fn test_oai_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- OAI endpoints allow user role (not 401/403) ---
// Only testing list_models and get_model here because they use the real DataService.
// chat_completions and embeddings use MockSharedContext which panics without expectations,
// but they share the same user_apis route_layer so the auth behavior is identical.

#[rstest]
#[case::list_models("GET", "/v1/models")]
#[case::get_model("GET", "/v1/models/some_model")]
#[tokio::test]
async fn test_oai_endpoints_allow_user(#[case] method: &str, #[case] path: &str) {
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
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED,
    status,
    "user should not get 401 for {method} {path}"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "user should not get 403 for {method} {path}"
  );
}
