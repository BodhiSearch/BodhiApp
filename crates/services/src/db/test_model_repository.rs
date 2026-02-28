use crate::{
  db::{DownloadRequest, DownloadStatus, ModelMetadataRow, ModelRepository},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use objs::{ApiAlias, ApiFormat};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// ===== Download Request Tests =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_download_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let request = DownloadRequest::new_pending("test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  let fetched = ctx.service.get_download_request(&request.id).await?;
  assert!(fetched.is_some());
  assert_eq!(request, fetched.unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_download_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut request = DownloadRequest::new_pending("test/repo", "test_file.gguf", ctx.now);
  ctx.service.create_download_request(&request).await?;
  request.status = DownloadStatus::Completed;
  request.total_bytes = Some(1000000);
  request.downloaded_bytes = 1000000;
  request.started_at = Some(ctx.now);
  request.updated_at = ctx.now + chrono::Duration::hours(1);
  ctx.service.update_download_request(&request).await?;

  let fetched = ctx
    .service
    .get_download_request(&request.id)
    .await?
    .unwrap();
  assert_eq!(DownloadStatus::Completed, fetched.status);
  assert_eq!(Some(1000000), fetched.total_bytes);
  assert_eq!(1000000, fetched.downloaded_bytes);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_download_requests(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  for i in 0..5 {
    let request = DownloadRequest::new_pending("test/repo", &format!("file_{}.gguf", i), ctx.now);
    ctx.service.create_download_request(&request).await?;
  }
  let (items, total) = ctx.service.list_download_requests(1, 3).await?;
  assert_eq!(3, items.len());
  assert_eq!(5, total);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_find_download_by_repo_filename(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let r1 = DownloadRequest::new_pending("test/repo", "file.gguf", ctx.now);
  let r2 = DownloadRequest::new_pending("test/repo", "file.gguf", ctx.now);
  let r3 = DownloadRequest::new_pending("other/repo", "file.gguf", ctx.now);
  ctx.service.create_download_request(&r1).await?;
  ctx.service.create_download_request(&r2).await?;
  ctx.service.create_download_request(&r3).await?;

  let results = ctx
    .service
    .find_download_request_by_repo_filename("test/repo", "file.gguf")
    .await?;
  assert_eq!(2, results.len());
  Ok(())
}

// ===== API Model Alias Tests =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_api_model_alias_crud(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias = ApiAlias {
    id: ulid::Ulid::new().to_string(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com".to_string(),
    models: vec!["gpt-4".to_string()].into(),
    prefix: Some("test".to_string()),
    forward_all_with_prefix: false,
    models_cache: vec![].into(),
    cache_fetched_at: ctx.now,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias(&alias, Some("sk-test-key".to_string()))
    .await?;

  let fetched = ctx.service.get_api_model_alias(&alias.id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(alias.id, fetched.id);
  assert_eq!(alias.base_url, fetched.base_url);

  let api_key = ctx.service.get_api_key_for_alias(&alias.id).await?;
  assert_eq!(Some("sk-test-key".to_string()), api_key);

  ctx.service.delete_api_model_alias(&alias.id).await?;
  let deleted = ctx.service.get_api_model_alias(&alias.id).await?;
  assert!(deleted.is_none());
  Ok(())
}

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
    source: "model".to_string(),
    repo: Some("test/repo".to_string()),
    filename: Some("model.gguf".to_string()),
    snapshot: Some("main".to_string()),
    api_model_id: None,
    capabilities: Some(objs::ModelCapabilities {
      vision: Some(true),
      audio: None,
      thinking: None,
      tools: objs::ToolCapabilities {
        function_calling: Some(true),
        structured_output: None,
      },
    }),
    context: Some(objs::ContextLimits {
      max_input_tokens: Some(4096),
      max_output_tokens: Some(2048),
    }),
    architecture: Some(objs::ModelArchitecture {
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
  assert_eq!("model", fetched.source);
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
      source: "model".to_string(),
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
      source: "model".to_string(),
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
