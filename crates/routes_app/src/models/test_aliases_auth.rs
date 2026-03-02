use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use rstest::rstest;
use tower::ServiceExt;

// Auth tier tests (merged from tests/routes_models_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_models("GET", "/bodhi/v1/models")]
#[case::get_model("GET", "/bodhi/v1/models/some-id")]
#[case::list_modelfiles("GET", "/bodhi/v1/modelfiles")]
#[tokio::test]
async fn test_read_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_read_endpoints_allow_all_roles(
  #[values(
    "resource_user",
    "resource_power_user",
    "resource_manager",
    "resource_admin"
  )]
  role: &str,
  #[values(
    ("GET", "/bodhi/v1/models"),
    ("GET", "/bodhi/v1/modelfiles")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  // Both endpoints return 200 OK with empty list from real DataService
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case::create_model("POST", "/bodhi/v1/models")]
#[case::update_model("PUT", "/bodhi/v1/models/some-id")]
#[case::delete_model("DELETE", "/bodhi/v1/models/some-id")]
#[case::copy_model("POST", "/bodhi/v1/models/some-id/copy")]
#[tokio::test]
async fn test_write_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_write_endpoints_reject_user_role(
  #[values(
    ("POST", "/bodhi/v1/models"),
    ("PUT", "/bodhi/v1/models/some-id"),
    ("DELETE", "/bodhi/v1/models/some-id"),
    ("POST", "/bodhi/v1/models/some-id/copy")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_write_endpoints_allow_power_user_and_above(
  #[values("resource_power_user", "resource_manager", "resource_admin")] role: &str,
  #[values(
    ("DELETE", "/bodhi/v1/models/some-id")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  // DELETE returns 404 for non-existent model (proves auth passed)
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}
