use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::ResponseTestExt;
use tower::ServiceExt;

// Auth tiers: consent GET/POST live in guest_endpoints (session auth, Guest floor —
// approve enforces User+ in-handler, covered in test_access_request.rs); list + revoke
// are session-auth user endpoints.

#[anyhow_trace]
#[rstest]
#[case::consent_context("GET", "/bodhi/v1/apps/access-requests/consent")]
#[case::submit_consent("POST", "/bodhi/v1/apps/access-requests")]
#[case::list_app_access("GET", "/bodhi/v1/access-requests/apps")]
#[case::revoke("POST", "/bodhi/v1/access-requests/test-id/revoke")]
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

#[anyhow_trace]
#[rstest]
#[case::guest_no_roles(&[])]
#[case::user(&["resource_user"])]
#[tokio::test]
async fn test_consent_context_reachable_by_guest_session(
  #[case] roles: &[&str],
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), roles).await?;
  // No query → an in-app error union, but the tier admits the session (200, not 401/403).
  // Safe with build_test_router: the missing client_id fails before MockAuthService is hit.
  let response = router
    .oneshot(session_request(
      "GET",
      "/bodhi/v1/apps/access-requests/consent",
      &cookie,
    ))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!("error", body["result"].as_str().unwrap());
  assert_eq!("invalid_request", body["error"].as_str().unwrap());
  Ok(())
}

// NOTE: no full-router "allow" test for list/revoke — build_test_router wires a
// MockAccessRequestService with no expectations; positive coverage lives in
// test_access_request.rs through the real DefaultAccessRequestService harness.
