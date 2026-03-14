use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[anyhow_trace]
#[rstest]
#[case::get_api_model("GET", "/bodhi/v1/models/api/some_id")]
#[case::create_api_model("POST", "/bodhi/v1/models/api")]
#[case::update_api_model("PUT", "/bodhi/v1/models/api/some_id")]
#[case::delete_api_model("DELETE", "/bodhi/v1/models/api/some_id")]
#[case::sync_models("POST", "/bodhi/v1/models/api/sync-models")]
#[case::test_api_model("POST", "/bodhi/v1/models/api/test")]
#[case::fetch_models("POST", "/bodhi/v1/models/api/fetch-models")]
#[case::list_api_formats("GET", "/bodhi/v1/models/api/formats")]
#[tokio::test]
async fn test_api_models_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_api_models_endpoints_allow_user_and_above(
  #[values(
    "resource_user",
    "resource_power_user",
    "resource_manager",
    "resource_admin"
  )]
  role: &str,
  #[values(
    ("GET", "/bodhi/v1/models/api/non-existent-id"),
    ("GET", "/bodhi/v1/models/api/formats")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  // May return 200 OK (formats) or 404 Not Found (non-existent ID)
  assert!(
    response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
    "Expected 200 or 404, got {} for {role} {method} {path}",
    response.status(),
  );
  Ok(())
}
