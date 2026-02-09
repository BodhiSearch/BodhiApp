use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Unauthenticated access rejected (401) ---

#[rstest]
#[case::list_api_models("GET", "/bodhi/v1/api-models")]
#[case::create_api_model("POST", "/bodhi/v1/api-models")]
#[case::get_api_model("GET", "/bodhi/v1/api-models/some_id")]
#[case::update_api_model("PUT", "/bodhi/v1/api-models/some_id")]
#[case::delete_api_model("DELETE", "/bodhi/v1/api-models/some_id")]
#[case::sync_models("POST", "/bodhi/v1/api-models/some_id/sync-models")]
#[case::test_api_model("POST", "/bodhi/v1/api-models/test")]
#[case::fetch_models("POST", "/bodhi/v1/api-models/fetch-models")]
#[case::get_api_formats("GET", "/bodhi/v1/api-models/api-formats")]
#[tokio::test]
async fn test_api_models_endpoints_reject_unauthenticated(
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

// --- Insufficient role rejected (403) ---

#[rstest]
#[case::resource_user("resource_user")]
#[tokio::test]
async fn test_api_models_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints: Vec<(&str, &str)> = vec![
    ("GET", "/bodhi/v1/api-models"),
    ("POST", "/bodhi/v1/api-models"),
    ("GET", "/bodhi/v1/api-models/some_id"),
    ("PUT", "/bodhi/v1/api-models/some_id"),
    ("DELETE", "/bodhi/v1/api-models/some_id"),
    ("POST", "/bodhi/v1/api-models/some_id/sync-models"),
    ("POST", "/bodhi/v1/api-models/test"),
    ("POST", "/bodhi/v1/api-models/fetch-models"),
    ("GET", "/bodhi/v1/api-models/api-formats"),
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
// Only safe endpoints are tested here:
// - GET /bodhi/v1/api-models (list - uses db_service which is real)
// - GET /bodhi/v1/api-models/api-formats (static data, no service calls)
// Skipping endpoints that use ai_api_service (test, fetch-models, sync-models) because
// the mock would panic without expectations. Also skipping create/update/delete which
// need valid request bodies or existing records. Since all endpoints share the same
// power_user_apis route_layer (ResourceRole::PowerUser), these two tests sufficiently
// prove the auth middleware allows power_user access.

#[rstest]
#[case::list_api_models("GET", "/bodhi/v1/api-models")]
#[case::get_api_formats("GET", "/bodhi/v1/api-models/api-formats")]
#[tokio::test]
async fn test_api_models_endpoints_allow_power_user(#[case] method: &str, #[case] path: &str) {
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
