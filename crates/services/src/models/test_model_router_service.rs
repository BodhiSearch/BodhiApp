use crate::models::{
  ApiAlias, ApiFormat, DefaultModelRouterService, ModelRouterError, ModelRouterRequest,
  ModelRouterService, RouterTargetRequest, RoutingStrategyConfig,
};
use crate::test_utils::{
  openai_model, sea_context, setup_env, FrozenTimeService, TEST_TENANT_ID, TEST_USER_ID,
};
use crate::{DbService, LocalDataService, MockHubService};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::sync::Arc;

struct Harness {
  service: DefaultModelRouterService,
  db: Arc<dyn DbService>,
  now: chrono::DateTime<chrono::Utc>,
  // Keep the sqlite temp dir alive for the lifetime of the harness.
  _temp_dir: Option<tempfile::TempDir>,
}

async fn harness(db_type: &str) -> Harness {
  let ctx = sea_context(db_type).await;
  let now = ctx.now;
  let temp_dir = ctx._temp_dir;
  let db: Arc<dyn DbService> = Arc::new(ctx.service);
  let mut hub = MockHubService::new();
  hub.expect_list_model_aliases().returning(|| Ok(vec![]));
  let data = Arc::new(LocalDataService::new(Arc::new(hub), db.clone()));
  let service =
    DefaultModelRouterService::new(db.clone(), data, Arc::new(FrozenTimeService::default()));
  Harness {
    service,
    db,
    now,
    _temp_dir: temp_dir,
  }
}

async fn seed_api_alias(
  h: &Harness,
  id: &str,
  format: ApiFormat,
  prefix: &str,
) -> anyhow::Result<()> {
  let api = ApiAlias::new(
    id,
    "test-name",
    format,
    "https://upstream.example.com/v1",
    vec![openai_model("gpt-4")],
    Some(prefix.to_string()),
    false,
    h.now,
    None,
    None,
  );
  h.db
    .create_api_model_alias(
      TEST_TENANT_ID,
      TEST_USER_ID,
      &api,
      Some("sk-test".to_string()),
    )
    .await?;
  Ok(())
}

fn target(alias: &str, model: &str, enabled: bool) -> RouterTargetRequest {
  RouterTargetRequest {
    alias: alias.to_string(),
    model: model.to_string(),
    enabled,
    weight: None,
  }
}

fn request(alias: &str, targets: Vec<RouterTargetRequest>) -> ModelRouterRequest {
  ModelRouterRequest {
    alias: alias.to_string(),
    targets,
    strategy: RoutingStrategyConfig::default(),
  }
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let h = harness(db_type).await;
  seed_api_alias(&h, "oai", ApiFormat::OpenAI, "oai/").await?;

  let created = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("my-stack", vec![target("oai", "oai/gpt-4", true)]),
    )
    .await?;
  assert_eq!("my-stack", created.alias);
  assert_eq!("model_router", created.source);
  assert_eq!(1, created.targets.len());

  let fetched = h
    .service
    .get(TEST_TENANT_ID, TEST_USER_ID, &created.id)
    .await?;
  assert_eq!(created, fetched);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_zero_and_all_disabled_targets_allowed(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let h = harness(db_type).await;
  seed_api_alias(&h, "oai", ApiFormat::OpenAI, "oai/").await?;

  // zero targets
  h.service
    .create(TEST_TENANT_ID, TEST_USER_ID, request("empty", vec![]))
    .await?;

  // all-disabled targets
  h.service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("all-off", vec![target("oai", "oai/gpt-4", false)]),
    )
    .await?;
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_duplicate_name_rejected(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let h = harness(db_type).await;
  seed_api_alias(&h, "oai", ApiFormat::OpenAI, "oai/").await?;
  h.service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("dup", vec![target("oai", "oai/gpt-4", true)]),
    )
    .await?;
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("dup", vec![target("oai", "oai/gpt-4", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(err, ModelRouterError::AliasExists { .. }));
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_validation_errors(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let h = harness(db_type).await;
  seed_api_alias(&h, "oai", ApiFormat::OpenAI, "oai/").await?;
  seed_api_alias(&h, "gem", ApiFormat::Gemini, "gem/").await?;

  // missing reference
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("r1", vec![target("absent", "absent/x", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::ReferencedAliasNotFound { .. }
  ));

  // self reference
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("r2", vec![target("r2", "r2", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(err, ModelRouterError::SelfReference { .. }));

  // invalid pinned model (not matchable for the api alias)
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("r3", vec![target("oai", "wrong-prefix/gpt-4", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(err, ModelRouterError::InvalidPinnedModel { .. }));

  // format without chat-completions surface (gemini)
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("r4", vec![target("gem", "gem/gpt-4", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::TargetFormatUnsupported { .. }
  ));
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_nested_router_rejected(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let h = harness(db_type).await;
  seed_api_alias(&h, "oai", ApiFormat::OpenAI, "oai/").await?;
  h.service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("inner-router", vec![target("oai", "oai/gpt-4", true)]),
    )
    .await?;
  // A target referencing another model-router must be rejected.
  let err = h
    .service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      request("outer", vec![target("inner-router", "inner-router", true)]),
    )
    .await
    .unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::NestedRouterNotAllowed { .. }
  ));
  Ok(())
}
