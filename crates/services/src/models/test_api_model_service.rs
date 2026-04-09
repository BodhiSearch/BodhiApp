use crate::models::{
  ApiAliasRepository, ApiFormat, ApiKeyUpdate, ApiModelRequest, ApiModelService,
  DefaultApiModelService,
};
use crate::test_utils::{
  test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID, TEST_USER_ID,
};
use crate::MockAiApiService;
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::{sync::Arc, time::Duration};

macro_rules! wait_for_event {
  ($rx:expr, $event_name:expr, $timeout:expr) => {{
    loop {
      tokio::select! {
          event = $rx.recv() => {
              match event {
                  Ok(e) if e == $event_name => break true,
                  _ => continue
              }
          }
          _ = tokio::time::sleep($timeout) => break false
      }
    }
  }};
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_forward_all_triggers_cache_refresh(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut rx = db_service.subscribe();
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _| Ok(vec!["model-a".to_string(), "model-b".to_string()]));

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

  let event_received = wait_for_event!(rx, "update_api_model_cache", Duration::from_millis(500));
  assert!(
    event_received,
    "Timed out waiting for update_api_model_cache event"
  );

  // Verify cache was populated in DB
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  assert_eq!(
    vec!["model-a".to_string(), "model-b".to_string()],
    alias.models_cache.to_vec()
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_non_forward_all_does_not_trigger_cache_refresh(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai.expect_fetch_models().times(0);

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

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_forward_all_triggers_cache_refresh(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut rx = db_service.subscribe();
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _| Ok(vec!["remote-1".to_string(), "remote-2".to_string()]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  // First create a non-forward_all model
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

  // Now update to forward_all
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

  let event_received = wait_for_event!(rx, "update_api_model_cache", Duration::from_millis(500));
  assert!(
    event_received,
    "Timed out waiting for update_api_model_cache event"
  );

  // Verify cache was populated in DB
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  assert_eq!(
    vec!["remote-1".to_string(), "remote-2".to_string()],
    alias.models_cache.to_vec()
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_non_forward_all_does_not_trigger_cache_refresh(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai.expect_fetch_models().times(0);

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  // Create a non-forward_all model
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

  // Update without forward_all
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

  Ok(())
}
