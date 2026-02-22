use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use rstest::rstest;
use tower::ServiceExt;

#[anyhow_trace]
#[rstest]
#[case::app_review("GET", "/bodhi/v1/access-requests/test-id/review")]
#[case::app_approve("PUT", "/bodhi/v1/access-requests/test-id/approve")]
#[case::app_deny("POST", "/bodhi/v1/access-requests/test-id/deny")]
#[tokio::test]
async fn test_app_access_request_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}
