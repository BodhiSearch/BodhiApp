use axum::http::StatusCode;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_ping_returns_200() {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request("GET", "/ping"))
    .await
    .unwrap();
  assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn test_authenticated_endpoint_rejects_unauthenticated() {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request("GET", "/bodhi/v1/models"))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[tokio::test]
async fn test_authenticated_endpoint_allows_session_auth() {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_user"],
  )
  .await
  .unwrap();
  let response = router
    .oneshot(session_request("GET", "/bodhi/v1/models", &cookie))
    .await
    .unwrap();
  assert_ne!(StatusCode::UNAUTHORIZED, response.status());
  assert_ne!(StatusCode::FORBIDDEN, response.status());
}
