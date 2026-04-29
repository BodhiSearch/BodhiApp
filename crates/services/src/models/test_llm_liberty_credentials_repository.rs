use crate::models::llm_liberty_credentials_repository::LlmLibertyCredentialsRepository;
use crate::models::llm_liberty_envelope::{
  LlmLibertyApiEndpoints, LlmLibertyAuthSpec, LlmLibertyEnvelope, LlmLibertyOauthEndpoints,
};
use crate::models::{ApiAlias, ApiFormat, ApiModelVec};
use crate::test_utils::{
  sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
};
use crate::ApiAliasRepository;
use anyhow_trace::anyhow_trace;
use chrono::{Duration, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_envelope(access: &str, refresh: &str, expires_at: i64) -> LlmLibertyEnvelope {
  LlmLibertyEnvelope {
    version: "1.0.0".into(),
    provider: "anthropic".into(),
    access_token: access.into(),
    refresh_token: refresh.into(),
    expires_at,
    auth: LlmLibertyAuthSpec {
      location: "header".into(),
      key: "Authorization".into(),
      scheme: "Bearer".into(),
    },
    oauth: LlmLibertyOauthEndpoints {
      authorize_url: "https://oauth.example/authorize".into(),
      token_url: "https://oauth.example/token".into(),
      revoke_url: None,
      client_id: "client-id-public".into(),
      client_secret: None,
    },
    api: LlmLibertyApiEndpoints {
      base_url: "https://api.anthropic.com/v1".into(),
      chat_url: "https://api.anthropic.com/v1/messages".into(),
      models_url: Some("https://api.anthropic.com/v1/models".into()),
    },
    headers: serde_json::json!({"anthropic-version": "2023-06-01"}),
    body: serde_json::json!({}),
    extra: None,
  }
}

async fn create_alias(
  ctx: &crate::test_utils::SeaTestContext,
  tenant_id: &str,
  user_id: &str,
  id: &str,
) -> anyhow::Result<()> {
  let alias = ApiAlias::new(
    id,
    ApiFormat::LlmLibertyOauth,
    "https://api.anthropic.com/v1",
    ApiModelVec::default(),
    None,
    false,
    ctx.now,
    None,
    None,
  );
  ctx
    .service
    .create_api_model_alias(tenant_id, user_id, &alias, None)
    .await?;
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_then_get_round_trip_decrypts(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-1";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let envelope = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  let resolved = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?
    .expect("credentials row");
  assert_eq!("access-1", resolved.access_token);
  assert_eq!("refresh-1", resolved.refresh_token);
  assert_eq!("https://api.anthropic.com/v1", resolved.api_base_url);
  assert_eq!("client-id-public", resolved.oauth_client_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_credentials_replaces_atomically(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-2";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let env_v1 = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &env_v1)
    .await?;

  let env_v2 = make_envelope("access-2", "refresh-2", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .update_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &env_v2)
    .await?;

  let resolved = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?
    .expect("credentials row");
  assert_eq!("access-2", resolved.access_token);
  assert_eq!("refresh-2", resolved.refresh_token);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_tokens_only_changes_token_columns(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-3";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let envelope = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  let new_expires = Utc::now() + Duration::hours(2);
  ctx
    .service
    .update_llm_liberty_tokens(
      TEST_TENANT_ID,
      alias_id,
      "access-rotated",
      "refresh-rotated",
      new_expires,
    )
    .await?;

  let resolved = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?
    .expect("credentials row");
  assert_eq!("access-rotated", resolved.access_token);
  assert_eq!("refresh-rotated", resolved.refresh_token);
  // Non-token columns must remain untouched.
  assert_eq!("https://oauth.example/token", resolved.oauth_token_url);
  assert_eq!("client-id-public", resolved.oauth_client_id);
  assert_eq!("https://api.anthropic.com/v1", resolved.api_base_url);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_returns_none_for_other_tenant(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-isolation-1";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let envelope = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  // Same alias_id, different tenant — RLS / scope check returns None.
  let cross = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_B_ID, TEST_USER_ID, alias_id)
    .await?;
  assert_eq!(None, cross.map(|c| c.access_token));
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_returns_none_for_other_user_same_tenant(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-isolation-2";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let envelope = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  let cross = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, alias_id)
    .await?;
  assert_eq!(None, cross.map(|c| c.access_token));
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_cascades_on_alias_delete(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-cascade";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let envelope = make_envelope("access-1", "refresh-1", (Utc::now() + Duration::hours(8)).timestamp());
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  // Sanity check
  assert!(ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?
    .is_some());

  // Delete the alias — FK CASCADE should drop the credentials row.
  ctx
    .service
    .delete_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?;

  let after = ctx
    .service
    .get_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?;
  assert_eq!(None, after.map(|c| c.access_token));
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_summary_returns_non_secret_fields_only(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_id = "alias-rt-summary";
  create_alias(&ctx, TEST_TENANT_ID, TEST_USER_ID, alias_id).await?;

  let expires_at_secs = (Utc::now() + Duration::hours(8)).timestamp();
  let envelope = make_envelope("access-1", "refresh-1", expires_at_secs);
  ctx
    .service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &envelope)
    .await?;

  let summary = ctx
    .service
    .get_llm_liberty_summary(TEST_TENANT_ID, TEST_USER_ID, alias_id)
    .await?
    .expect("summary");
  assert_eq!("anthropic", summary.provider);
  assert_eq!("1.0.0", summary.envelope_version);
  assert!(summary.has_refresh_token);
  Ok(())
}
