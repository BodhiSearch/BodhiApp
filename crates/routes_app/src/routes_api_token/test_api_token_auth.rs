use anyhow_trace::anyhow_trace;
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use tower::ServiceExt;

#[anyhow_trace]
#[rstest]
#[case::list_tokens("GET", "/bodhi/v1/tokens")]
#[case::create_token("POST", "/bodhi/v1/tokens")]
#[case::update_token("PUT", "/bodhi/v1/tokens/some_token_id")]
#[tokio::test]
async fn test_token_endpoints_reject_unauthenticated(
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
async fn test_token_endpoints_reject_insufficient_role(
  #[values("resource_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/tokens"),
    ("POST", "/bodhi/v1/tokens"),
    ("PUT", "/bodhi/v1/tokens/some_token_id")
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
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_token_list_endpoint_allows_eligible_roles(
  #[values("resource_power_user", "resource_manager", "resource_admin")] role: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router
    .oneshot(session_request("GET", "/bodhi/v1/tokens", &cookie))
    .await?;
  // GET /bodhi/v1/tokens returns 200 with empty list from real DbService
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
