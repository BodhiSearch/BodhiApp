use crate::{
  models::{
    AliasSource, ContextLimits, ModelArchitecture, ModelCapabilities, ModelMetadataRepository,
    ModelMetadataRow, ToolCapabilities,
  },
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// ===== Model Metadata Tests =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_upsert_and_get_model_metadata(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let metadata = ModelMetadataRow {
    id: String::new(),
    source: AliasSource::Model,
    repo: Some("test/repo".to_string()),
    filename: Some("model.gguf".to_string()),
    snapshot: Some("main".to_string()),
    api_model_id: None,
    capabilities: Some(ModelCapabilities {
      vision: Some(true),
      audio: None,
      thinking: None,
      tools: ToolCapabilities {
        function_calling: Some(true),
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
    chat_template: Some("template".to_string()),
    extracted_at: ctx.now,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx.service.upsert_model_metadata(&metadata).await?;

  let fetched = ctx
    .service
    .get_model_metadata_by_file("test/repo", "model.gguf", "main")
    .await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(AliasSource::Model, fetched.source);
  assert_eq!(Some("test/repo".to_string()), fetched.repo);
  assert_eq!(
    Some(true),
    fetched.capabilities.as_ref().and_then(|c| c.vision)
  );
  assert_eq!(
    Some(4096),
    fetched.context.as_ref().and_then(|c| c.max_input_tokens)
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_batch_get_metadata(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let files = vec![
    (
      "repo1".to_string(),
      "file1.gguf".to_string(),
      "main".to_string(),
    ),
    (
      "repo2".to_string(),
      "file2.gguf".to_string(),
      "main".to_string(),
    ),
    (
      "repo3".to_string(),
      "file3.gguf".to_string(),
      "main".to_string(),
    ),
  ];

  for (repo, filename, snapshot) in &files {
    let metadata = ModelMetadataRow {
      id: String::new(),
      source: AliasSource::Model,
      repo: Some(repo.clone()),
      filename: Some(filename.clone()),
      snapshot: Some(snapshot.clone()),
      api_model_id: None,
      extracted_at: ctx.now,
      created_at: ctx.now,
      updated_at: ctx.now,
      ..Default::default()
    };
    ctx.service.upsert_model_metadata(&metadata).await?;
  }

  let result = ctx.service.batch_get_metadata_by_files(&files).await?;
  assert_eq!(3, result.len());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_model_metadata(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  for i in 0..3 {
    let metadata = ModelMetadataRow {
      id: String::new(),
      source: AliasSource::Model,
      repo: Some(format!("repo{}", i)),
      filename: Some(format!("file{}.gguf", i)),
      snapshot: Some("main".to_string()),
      api_model_id: None,
      extracted_at: ctx.now,
      created_at: ctx.now,
      updated_at: ctx.now,
      ..Default::default()
    };
    ctx.service.upsert_model_metadata(&metadata).await?;
  }

  let result = ctx.service.list_model_metadata().await?;
  assert_eq!(3, result.len());
  Ok(())
}
