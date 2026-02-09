use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Ollama endpoints (User-tier): unauthenticated access rejected (401) ---

#[rstest]
#[case::list_tags("GET", "/api/tags")]
#[case::show_model("POST", "/api/show")]
#[case::chat("POST", "/api/chat")]
#[tokio::test]
async fn test_ollama_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Ollama endpoints allow user role (not 401/403) ---
// Only testing list_tags (GET /api/tags) here because it uses the real DataService
// and requires no request body. show_model (POST /api/show) needs a valid JSON body,
// and chat (POST /api/chat) uses MockSharedContext which panics without expectations.
// All three share the same user_apis route_layer so the auth behavior is identical.

#[rstest]
#[case::list_tags("GET", "/api/tags")]
#[tokio::test]
async fn test_ollama_endpoints_allow_user(#[case] method: &str, #[case] path: &str) {
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
