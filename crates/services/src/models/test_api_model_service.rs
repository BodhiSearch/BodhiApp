use crate::models::{
  ApiAliasRepository, ApiFormat, ApiKeyUpdate, ApiModel, ApiModelRequest, ApiModelService,
  DefaultApiModelService,
};
use crate::test_utils::{
  openai_model, test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID, TEST_USER_ID,
};
use crate::MockAiApiService;
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_forward_all_stores_all_models(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _| Ok(vec![openai_model("model-a"), openai_model("model-b")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("test".to_string()),
    forward_all_with_prefix: true,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert!(result.forward_all_with_prefix);
  assert_eq!(
    vec![openai_model("model-a"), openai_model("model-b")],
    result.models
  );

  // Verify models stored in DB
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  assert_eq!(
    vec![openai_model("model-a"), openai_model("model-b")],
    alias.models.to_vec()
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_non_forward_all_validates_and_filters(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _| Ok(vec![openai_model("gpt-4"), openai_model("gpt-3.5")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert!(!result.forward_all_with_prefix);
  assert_eq!(vec![openai_model("gpt-4")], result.models);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_forward_all_stores_all_models(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  // create() fetches once, update() fetches once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(|_, _, _| Ok(vec![openai_model("remote-1"), openai_model("remote-2")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("initial".to_string()),
    forward_all_with_prefix: true,
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("test".to_string()),
    forward_all_with_prefix: true,
  };
  let result = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await?;
  assert!(result.forward_all_with_prefix);
  assert_eq!(
    vec![openai_model("remote-1"), openai_model("remote-2")],
    result.models
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_non_forward_all_validates_and_filters(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  // create() calls fetch_models once, update() calls fetch_models once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(|_, _, _| Ok(vec![openai_model("gpt-4"), openai_model("gpt-3.5")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com/v2".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string(), "gpt-3.5".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };
  let result = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await?;
  assert!(!result.forward_all_with_prefix);
  let mut model_ids: Vec<&str> = result.models.iter().map(|m| m.id()).collect();
  model_ids.sort();
  assert_eq!(vec!["gpt-3.5", "gpt-4"], model_ids);

  Ok(())
}
