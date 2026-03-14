use crate::test_utils::{build_test_router, cors_preflight_request, request_with_origin};
use rstest::rstest;
use tower::ServiceExt;

/// Test that CORS preflight requests to session-only endpoints
/// do NOT return Access-Control-Allow-Origin (restrictive CORS).
#[rstest]
#[case::session_tokens_post("/bodhi/v1/tokens", "POST")]
#[case::session_tokens_get("/bodhi/v1/tokens", "GET")]
#[case::session_settings_get("/bodhi/v1/settings", "GET")]
#[case::session_users_get("/bodhi/v1/users", "GET")]
#[case::session_toolset_types_get("/bodhi/v1/toolset_types", "GET")]
#[case::session_queue_get("/bodhi/v1/queue", "GET")]
#[case::session_toolsets_get("/bodhi/v1/toolsets", "GET")]
#[case::session_toolsets_post("/bodhi/v1/toolsets", "POST")]
#[case::session_mcps_get("/bodhi/v1/mcps", "GET")]
#[case::session_mcps_post("/bodhi/v1/mcps", "POST")]
#[tokio::test]
async fn test_cors_preflight_session_endpoints_blocked(
  #[case] path: &str,
  #[case] method: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _temp) = build_test_router().await?;
  let req = cors_preflight_request(path, method, "https://evil.com");
  let response = router.oneshot(req).await?;

  assert!(
    response
      .headers()
      .get("access-control-allow-origin")
      .is_none(),
    "Session endpoint {} should NOT have Access-Control-Allow-Origin header",
    path
  );
  Ok(())
}

/// Test that CORS preflight requests to non-session API endpoints
/// DO return Access-Control-Allow-Origin: * (permissive CORS).
#[rstest]
#[case::oai_models_get("/v1/models", "GET")]
#[case::oai_chat_completions_post("/v1/chat/completions", "POST")]
#[case::ping_get("/ping", "GET")]
#[case::health_get("/health", "GET")]
#[case::info_get("/bodhi/v1/info", "GET")]
#[case::models_get("/bodhi/v1/models", "GET")]
#[case::apps_toolsets_get("/bodhi/v1/apps/toolsets", "GET")]
#[case::apps_mcps_get("/bodhi/v1/apps/mcps", "GET")]
#[tokio::test]
async fn test_cors_preflight_nonsession_endpoints_allowed(
  #[case] path: &str,
  #[case] method: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _temp) = build_test_router().await?;
  let req = cors_preflight_request(path, method, "https://example.com");
  let response = router.oneshot(req).await?;

  let acao = response
    .headers()
    .get("access-control-allow-origin")
    .expect(&format!(
      "Non-session endpoint {} should have Access-Control-Allow-Origin header",
      path
    ));
  assert_eq!(
    "*",
    acao.to_str().unwrap(),
    "Non-session endpoint {} should allow any origin",
    path
  );
  Ok(())
}

/// Test that non-preflight requests to session endpoints do NOT get CORS headers.
#[rstest]
#[case::session_tokens_get("/bodhi/v1/tokens", "GET")]
#[case::session_settings_get("/bodhi/v1/settings", "GET")]
#[tokio::test]
async fn test_cors_non_preflight_session_endpoints_no_cors(
  #[case] path: &str,
  #[case] method: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _temp) = build_test_router().await?;
  let req = request_with_origin(method, path, "https://evil.com");
  let response = router.oneshot(req).await?;

  assert!(
    response
      .headers()
      .get("access-control-allow-origin")
      .is_none(),
    "Session endpoint {} should NOT have Access-Control-Allow-Origin on non-preflight requests",
    path
  );
  Ok(())
}

/// Test that non-preflight requests to non-session endpoints DO get CORS headers.
#[rstest]
#[case::oai_models_get("/v1/models", "GET")]
#[case::ping_get("/ping", "GET")]
#[tokio::test]
async fn test_cors_non_preflight_nonsession_endpoints_allowed(
  #[case] path: &str,
  #[case] method: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _temp) = build_test_router().await?;
  let req = request_with_origin(method, path, "https://example.com");
  let response = router.oneshot(req).await?;

  let acao = response
    .headers()
    .get("access-control-allow-origin")
    .expect(&format!(
      "Non-session endpoint {} should have Access-Control-Allow-Origin on non-preflight",
      path
    ));
  assert_eq!(
    "*",
    acao.to_str().unwrap(),
    "Non-session endpoint {} should allow any origin",
    path
  );
  Ok(())
}
