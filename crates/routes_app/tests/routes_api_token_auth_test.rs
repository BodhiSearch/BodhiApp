use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Unauthenticated access rejected (401) ---

#[rstest]
#[case::list_tokens("GET", "/bodhi/v1/tokens")]
#[case::create_token("POST", "/bodhi/v1/tokens")]
#[case::update_token("PUT", "/bodhi/v1/tokens/some_token_id")]
#[tokio::test]
async fn test_token_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Insufficient role rejected (403) ---

#[rstest]
#[case::resource_user("resource_user")]
#[tokio::test]
async fn test_token_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints: Vec<(&str, &str)> = vec![
    ("GET", "/bodhi/v1/tokens"),
    ("POST", "/bodhi/v1/tokens"),
    ("PUT", "/bodhi/v1/tokens/some_token_id"),
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

// --- PowerUser allowed (not 401/403) ---
// Only list_tokens is tested here because it safely uses db_service with no setup required.
// create_token needs a valid JSON body with token scope/role fields, and update_token needs
// a valid token_id that exists in the database. Since all three endpoints share the same
// power_user_session_apis route_layer (ResourceRole::PowerUser), the list_tokens test
// sufficiently proves the auth middleware allows power_user access.

#[rstest]
#[case::list_tokens("GET", "/bodhi/v1/tokens")]
#[tokio::test]
async fn test_token_endpoints_allow_power_user(#[case] method: &str, #[case] path: &str) {
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
