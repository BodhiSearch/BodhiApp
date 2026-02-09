use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

// --- Unauthenticated access rejected (401) ---

#[rstest]
#[case::list_settings("GET", "/bodhi/v1/settings")]
#[case::update_setting("PUT", "/bodhi/v1/settings/some_key")]
#[case::delete_setting("DELETE", "/bodhi/v1/settings/some_key")]
#[case::list_toolset_types("GET", "/bodhi/v1/toolset_types")]
#[case::enable_toolset_type("PUT", "/bodhi/v1/toolset_types/some_type/app-config")]
#[case::disable_toolset_type("DELETE", "/bodhi/v1/toolset_types/some_type/app-config")]
#[tokio::test]
async fn test_admin_endpoints_reject_unauthenticated(
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
#[case::user("resource_user")]
#[case::power_user("resource_power_user")]
#[tokio::test]
async fn test_admin_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints: Vec<(&str, &str)> = vec![
    ("GET", "/bodhi/v1/settings"),
    ("PUT", "/bodhi/v1/settings/some_key"),
    ("DELETE", "/bodhi/v1/settings/some_key"),
    ("GET", "/bodhi/v1/toolset_types"),
    ("PUT", "/bodhi/v1/toolset_types/some_type/app-config"),
    ("DELETE", "/bodhi/v1/toolset_types/some_type/app-config"),
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

// --- Admin allowed (not 401/403) ---
// Settings endpoints use real SettingService stub, so admin requests reach the handler
// and return a non-auth response. Toolset type endpoints are excluded here because
// their handlers call MockToolService (which panics with no expectations set).
// Since all 6 endpoints share the same admin_session_apis route_layer (ResourceRole::Admin),
// the settings tests sufficiently prove the auth middleware allows admin access.

#[rstest]
#[case::list_settings("GET", "/bodhi/v1/settings")]
#[case::update_setting("PUT", "/bodhi/v1/settings/some_key")]
#[case::delete_setting("DELETE", "/bodhi/v1/settings/some_key")]
#[tokio::test]
async fn test_admin_endpoints_allow_admin(
  #[case] method: &str,
  #[case] path: &str,
) {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(),
    &["resource_admin"],
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
    "admin should not get 401 for {method} {path}"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "admin should not get 403 for {method} {path}"
  );
}
