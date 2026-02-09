use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Read endpoints (User-tier): unauthenticated access rejected (401) ---

#[rstest]
#[case::list_models("GET", "/bodhi/v1/models")]
#[case::get_model("GET", "/bodhi/v1/models/some-id")]
#[case::list_modelfiles("GET", "/bodhi/v1/modelfiles")]
#[tokio::test]
async fn test_read_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Read endpoints allow user role (not 401/403) ---

#[rstest]
#[case::list_models("GET", "/bodhi/v1/models")]
#[case::get_model("GET", "/bodhi/v1/models/some-id")]
#[case::list_modelfiles("GET", "/bodhi/v1/modelfiles")]
#[tokio::test]
async fn test_read_endpoints_allow_user(#[case] method: &str, #[case] path: &str) {
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

// --- Write endpoints (PowerUser-tier): unauthenticated access rejected (401) ---

#[rstest]
#[case::create_model("POST", "/bodhi/v1/models")]
#[case::update_model("PUT", "/bodhi/v1/models/some-id")]
#[case::delete_model("DELETE", "/bodhi/v1/models/some-id")]
#[case::copy_model("POST", "/bodhi/v1/models/some-id/copy")]
#[tokio::test]
async fn test_write_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// --- Write endpoints reject user role (403) ---

#[rstest]
#[case::create_model("POST", "/bodhi/v1/models")]
#[case::update_model("PUT", "/bodhi/v1/models/some-id")]
#[case::delete_model("DELETE", "/bodhi/v1/models/some-id")]
#[case::copy_model("POST", "/bodhi/v1/models/some-id/copy")]
#[tokio::test]
async fn test_write_endpoints_reject_user_role(#[case] method: &str, #[case] path: &str) {
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

// --- Write endpoints allow power_user role (not 401/403) ---

#[rstest]
#[case::create_model("POST", "/bodhi/v1/models")]
#[case::update_model("PUT", "/bodhi/v1/models/some-id")]
#[case::delete_model("DELETE", "/bodhi/v1/models/some-id")]
#[case::copy_model("POST", "/bodhi/v1/models/some-id/copy")]
#[tokio::test]
async fn test_write_endpoints_allow_power_user(#[case] method: &str, #[case] path: &str) {
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
