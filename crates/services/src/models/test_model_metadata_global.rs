use crate::{
  models::{
    AliasSource, ContextLimits, ModelArchitecture, ModelCapabilities, ModelMetadataEntity,
    ModelMetadataRepository, ToolCapabilities,
  },
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_metadata(
  tenant_id: &str,
  repo: &str,
  filename: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> ModelMetadataEntity {
  ModelMetadataEntity {
    id: String::new(),
    tenant_id: tenant_id.to_string(),
    source: AliasSource::Model,
    repo: Some(repo.to_string()),
    filename: Some(filename.to_string()),
    snapshot: Some("main".to_string()),
    api_model_id: None,
    capabilities: Some(ModelCapabilities {
      vision: Some(true),
      audio: None,
      thinking: None,
      tools: ToolCapabilities {
        function_calling: None,
        structured_output: None,
      },
    }),
    context: Some(ContextLimits {
      max_input_tokens: Some(4096),
      max_output_tokens: Some(2048),
    }),
    architecture: Some(ModelArchitecture {
      family: None,
      parameter_count: None,
      quantization: None,
      format: "llama".to_string(),
    }),
    additional_metadata: None,
    chat_template: None,
    extracted_at: now,
    created_at: now,
    updated_at: now,
  }
}

/// Model metadata is tenant-scoped: each tenant has its own metadata entries.
/// Verify cross-tenant isolation: metadata in tenant A is not visible to tenant B.
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_model_metadata_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let meta_a = make_metadata(TEST_TENANT_ID, "test/repo-a", "model-a.gguf", ctx.now);
  let meta_b = make_metadata(TEST_TENANT_B_ID, "test/repo-b", "model-b.gguf", ctx.now);

  ctx.service.upsert_model_metadata(&meta_a).await?;
  ctx.service.upsert_model_metadata(&meta_b).await?;

  // Tenant A only sees its own metadata
  let list_a = ctx.service.list_model_metadata(TEST_TENANT_ID).await?;
  assert_eq!(1, list_a.len());
  assert_eq!(TEST_TENANT_ID, list_a[0].tenant_id);

  // Tenant B only sees its own metadata
  let list_b = ctx.service.list_model_metadata(TEST_TENANT_B_ID).await?;
  assert_eq!(1, list_b.len());
  assert_eq!(TEST_TENANT_B_ID, list_b[0].tenant_id);

  // Cross-tenant get returns None
  let cross = ctx
    .service
    .get_model_metadata_by_file(TEST_TENANT_ID, "test/repo-b", "model-b.gguf", "main")
    .await?;
  assert!(cross.is_none());

  Ok(())
}
