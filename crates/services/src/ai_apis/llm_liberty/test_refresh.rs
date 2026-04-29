use super::refresh::{ensure_fresh_credentials, LlmLibertyRefreshError};
use crate::models::llm_liberty_credentials_repository::MockLlmLibertyCredentialsRepository;
use crate::models::llm_liberty_envelope::ResolvedLlmLibertyCredentials;
use crate::SafeReqwest;
use anyhow_trace::anyhow_trace;
use chrono::{Duration, Utc};
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;

const TENANT_A: &str = "tenant-a";
const USER_A: &str = "user-a";

fn make_creds(token_url: &str, expires_at: chrono::DateTime<chrono::Utc>) -> ResolvedLlmLibertyCredentials {
  ResolvedLlmLibertyCredentials {
    access_token: "access-old".to_string(),
    refresh_token: "refresh-old".to_string(),
    expires_at,
    tenant_id: TENANT_A.to_string(),
    auth_scheme: "Bearer".to_string(),
    auth_key: "Authorization".to_string(),
    oauth_token_url: token_url.to_string(),
    oauth_client_id: "client-id-public".to_string(),
    oauth_client_secret: None,
    api_base_url: "https://api.anthropic.com/v1".to_string(),
    api_chat_url: "https://api.anthropic.com/v1/messages".to_string(),
    api_models_url: Some("https://api.anthropic.com/v1/models".to_string()),
    headers_json: serde_json::json!({}),
    body_json: serde_json::json!({}),
    extra_json: None,
  }
}

fn safe_http() -> SafeReqwest {
  SafeReqwest::builder()
    .allow_private_ips()
    .build()
    .expect("safe reqwest builder")
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_returns_creds_when_token_within_skew_window() -> anyhow::Result<()> {
  // Token expires 10 minutes from now — well outside the 60-second skew. No HTTP call.
  let mut server = Server::new_async().await;
  let token_url = format!("{}/oauth/token", server.url());
  let alias_id = "alias-skew-window";

  let mock = server
    .mock("POST", "/oauth/token")
    .expect(0)
    .with_status(200)
    .create_async()
    .await;

  let mut repo = MockLlmLibertyCredentialsRepository::new();
  let creds = make_creds(&token_url, Utc::now() + Duration::minutes(10));
  repo
    .expect_get_llm_liberty_credentials()
    .returning(move |_, _, _| Ok(Some(creds.clone())));
  repo
    .expect_update_llm_liberty_tokens()
    .times(0)
    .returning(|_, _, _, _, _| Ok(()));

  let result = ensure_fresh_credentials(&repo, &safe_http(), TENANT_A, USER_A, alias_id).await?;
  assert_eq!("access-old", result.access_token);
  mock.assert_async().await;
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_persists_rotated_tokens_when_expired() -> anyhow::Result<()> {
  // Token already expired — refresh path runs.
  let mut server = Server::new_async().await;
  let token_url = format!("{}/oauth/token", server.url());
  let alias_id = "alias-rotate";

  let mock = server
    .mock("POST", "/oauth/token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{"access_token":"access-new","refresh_token":"refresh-new","expires_in":28800}"#,
    )
    .create_async()
    .await;

  let mut repo = MockLlmLibertyCredentialsRepository::new();
  let creds = make_creds(&token_url, Utc::now() - Duration::minutes(5));
  repo
    .expect_get_llm_liberty_credentials()
    .returning(move |_, _, _| Ok(Some(creds.clone())));
  repo
    .expect_update_llm_liberty_tokens()
    .withf(|_, _, access, refresh, _| access == "access-new" && refresh == "refresh-new")
    .times(1)
    .returning(|_, _, _, _, _| Ok(()));

  let result = ensure_fresh_credentials(&repo, &safe_http(), TENANT_A, USER_A, alias_id).await?;
  assert_eq!("access-new", result.access_token);
  assert_eq!("refresh-new", result.refresh_token);
  mock.assert_async().await;
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_uses_old_refresh_token_when_response_omits_one() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let token_url = format!("{}/oauth/token", server.url());
  let alias_id = "alias-fallback-refresh";

  // Response omits refresh_token — caller should keep old.
  let _mock = server
    .mock("POST", "/oauth/token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"access_token":"access-new","expires_in":28800}"#)
    .create_async()
    .await;

  let mut repo = MockLlmLibertyCredentialsRepository::new();
  let creds = make_creds(&token_url, Utc::now() - Duration::minutes(5));
  repo
    .expect_get_llm_liberty_credentials()
    .returning(move |_, _, _| Ok(Some(creds.clone())));
  repo
    .expect_update_llm_liberty_tokens()
    .withf(|_, _, _access, refresh, _| refresh == "refresh-old")
    .times(1)
    .returning(|_, _, _, _, _| Ok(()));

  let result = ensure_fresh_credentials(&repo, &safe_http(), TENANT_A, USER_A, alias_id).await?;
  assert_eq!("refresh-old", result.refresh_token);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_concurrent_requests_serialize() -> anyhow::Result<()> {
  // Two concurrent ensure_fresh_credentials calls on the same alias must
  // serialize via the per-alias mutex: only one HTTP refresh observed.
  let mut server = Server::new_async().await;
  let token_url = format!("{}/oauth/token", server.url());
  let alias_id = "alias-mutex";

  let mock = server
    .mock("POST", "/oauth/token")
    .expect_at_most(1)
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{"access_token":"access-new","refresh_token":"refresh-new","expires_in":28800}"#,
    )
    .create_async()
    .await;

  // The first caller observes an expired token (drives a refresh). After the
  // mutex serializes the second caller and update_llm_liberty_tokens has
  // persisted, subsequent GETs return a fresh-enough token (skip refresh).
  let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let mut repo = MockLlmLibertyCredentialsRepository::new();
  {
    let cc = call_count.clone();
    let token_url = token_url.clone();
    repo
      .expect_get_llm_liberty_credentials()
      .returning(move |_, _, _| {
        let n = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let expires_at = if n == 0 {
          Utc::now() - Duration::minutes(5)
        } else {
          Utc::now() + Duration::minutes(30)
        };
        Ok(Some(make_creds(&token_url, expires_at)))
      });
  }
  repo
    .expect_update_llm_liberty_tokens()
    .times(1)
    .returning(|_, _, _, _, _| Ok(()));

  let http = safe_http();
  let r1 = ensure_fresh_credentials(&repo, &http, TENANT_A, USER_A, alias_id);
  let r2 = ensure_fresh_credentials(&repo, &http, TENANT_A, USER_A, alias_id);
  let (a, b) = tokio::join!(r1, r2);
  a?;
  b?;
  mock.assert_async().await;
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_propagates_5xx_as_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let token_url = format!("{}/oauth/token", server.url());
  let alias_id = "alias-5xx";

  let _mock = server
    .mock("POST", "/oauth/token")
    .with_status(503)
    .with_body("upstream unavailable")
    .create_async()
    .await;

  let mut repo = MockLlmLibertyCredentialsRepository::new();
  let creds = make_creds(&token_url, Utc::now() - Duration::minutes(5));
  repo
    .expect_get_llm_liberty_credentials()
    .returning(move |_, _, _| Ok(Some(creds.clone())));
  // Update must NOT be called when refresh fails.
  repo
    .expect_update_llm_liberty_tokens()
    .times(0)
    .returning(|_, _, _, _, _| Ok(()));

  let result = ensure_fresh_credentials(&repo, &safe_http(), TENANT_A, USER_A, alias_id).await;
  assert!(matches!(result, Err(LlmLibertyRefreshError::AiApi(_))));
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn refresh_returns_not_found_when_no_credentials_row() -> anyhow::Result<()> {
  let alias_id = "alias-missing";
  let mut repo = MockLlmLibertyCredentialsRepository::new();
  repo
    .expect_get_llm_liberty_credentials()
    .returning(|_, _, _| Ok(None));

  let result = ensure_fresh_credentials(&repo, &safe_http(), TENANT_A, USER_A, alias_id).await;
  assert!(matches!(result, Err(LlmLibertyRefreshError::NotFound(_))));
  Ok(())
}
