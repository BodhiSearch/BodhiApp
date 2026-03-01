use crate::models::{Alias, ApiAlias, ApiAliasRepository, ApiFormat};
use crate::{
  test_utils::{
    test_data_service, test_db_service, test_hf_service, TestDataService, TestDbService,
    TestHfService,
  },
  DataService, LocalDataService,
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_local_data_service_find_alias(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  let alias = service.find_alias("testalias-exists:instruct").await;
  assert!(alias.is_some());
  let alias = alias.unwrap();
  match &alias {
    Alias::User(user_alias) => {
      assert_eq!("testalias-exists:instruct", user_alias.alias);
    }
    _ => panic!("Expected User alias"),
  }
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_not_found(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  let alias = service.find_alias("nonexistent-alias").await;
  assert_eq!(None, alias);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_api_by_model_name(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);
  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  // Insert API alias with multiple models
  let api_alias = ApiAlias::new(
    "openai-api",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    None,
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  // Test finding by model name
  let found = data_service.find_alias("gpt-4").await;
  assert!(matches!(found, Some(Alias::Api(api)) if api.id == "openai-api"));

  let found = data_service.find_alias("gpt-3.5-turbo").await;
  assert!(matches!(found, Some(Alias::Api(api)) if api.id == "openai-api"));

  Ok(())
}

#[rstest]
#[case("testalias-exists:instruct", true, "user")] // User alias exists (seeded in DB)
#[case("gpt-4", true, "api")] // API model will be inserted
#[case("nonexistent-model", false, "none")] // Should not exist
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_priority_cases(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
  #[case] search_alias: &str,
  #[case] should_find: bool,
  #[case] expected_type: &str,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);

  // Seed user aliases into DB
  crate::test_utils::seed_test_user_aliases(db_service.as_ref()).await?;

  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  // Insert API alias with gpt-4 model
  let api_alias = ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  let found = data_service.find_alias(search_alias).await;

  if should_find {
    let alias = found.expect("Expected to find alias");
    match expected_type {
      "user" => assert!(matches!(alias, Alias::User(_))),
      "model" => assert!(matches!(alias, Alias::Model(_))),
      "api" => assert!(matches!(alias, Alias::Api(_))),
      _ => panic!("Invalid expected_type: {}", expected_type),
    }
  } else {
    assert!(found.is_none());
  }

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_user_priority_over_api(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);

  // Seed user aliases into DB
  crate::test_utils::seed_test_user_aliases(db_service.as_ref()).await?;

  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  // Insert API alias with model name that matches existing user alias
  let api_alias = ApiAlias::new(
    "conflicting-api",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["testalias-exists:instruct".to_string()], // Same name as user alias
    None,
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  // Should find user alias, not API alias (user has priority)
  let found = data_service.find_alias("testalias-exists:instruct").await;
  assert!(matches!(found, Some(Alias::User(_))));

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_local_data_service_list_aliases(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  let result = service.list_aliases().await?;
  // Should have at least the 3 seeded user aliases + model aliases from hub
  assert!(result.len() >= 3);
  // Check that user aliases are present
  assert!(result
    .iter()
    .any(|a| matches!(a, Alias::User(u) if u.alias == "llama3:instruct")));
  assert!(result
    .iter()
    .any(|a| matches!(a, Alias::User(u) if u.alias == "testalias-exists:instruct")));
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_local_data_service_delete_alias(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  // First, verify alias exists
  let alias = service.find_alias("tinyllama:instruct").await;
  assert!(alias.is_some());
  let id = match alias.unwrap() {
    Alias::User(u) => u.id,
    _ => panic!("Expected User alias"),
  };
  service.delete_alias(&id).await?;
  let alias = service.find_alias("tinyllama:instruct").await;
  assert!(alias.is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_local_data_service_delete_alias_not_found(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  let result = service.delete_alias("nonexistent-id").await;
  let err = result.unwrap_err();
  assert_eq!("data_service_error-alias_not_found", err.code());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_local_data_service_copy_alias(
  #[future]
  #[from(test_data_service)]
  service: TestDataService,
) -> anyhow::Result<()> {
  // Get the ID of the tinyllama alias
  let alias = service.find_alias("tinyllama:instruct").await;
  let id = match alias.unwrap() {
    Alias::User(u) => u.id,
    _ => panic!("Expected User alias"),
  };

  let new_alias = service.copy_alias(&id, "tinyllama:mymodel").await?;
  assert_eq!("tinyllama:mymodel", new_alias.alias);

  let found = service.find_user_alias("tinyllama:mymodel").await;
  assert!(found.is_some());
  let found = found.unwrap();
  assert_eq!("tinyllama:mymodel", found.alias);
  Ok(())
}

#[rstest]
#[case("azure/gpt-4", Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure-openai")]
#[case("gpt-4", None, vec!["gpt-4".to_string()], "legacy-api")]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_with_prefix_matches(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
  #[case] search_term: &str,
  #[case] api_prefix: Option<String>,
  #[case] api_models: Vec<String>,
  #[case] expected_id: &str,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);
  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  let test_alias = ApiAlias::new(
    expected_id,
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    api_models,
    api_prefix,
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&test_alias, Some("test-key".to_string()))
    .await?;

  let found = data_service.find_alias(search_term).await;
  let Some(Alias::Api(api)) = found else {
    panic!("Expected to find Api alias, but found none");
  };
  assert_eq!(expected_id, api.id);

  Ok(())
}

#[rstest]
#[case("non-matching-term")]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_with_non_matching_prefix_returns_none(
  #[case] search_term: &str,
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);
  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  let test_alias = ApiAlias::new(
    "azure-openai",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    Some("azure/".to_string()),
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&test_alias, Some("test-key".to_string()))
    .await?;

  let found = data_service.find_alias(search_term).await;
  assert!(found.is_none());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
#[awt]
async fn test_find_alias_without_prefix_does_not_match_prefixed_api(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(test_db_service);
  let data_service = LocalDataService::new(Arc::new(test_hf_service), db_service.clone());

  // Create API alias with prefix
  let prefixed_alias = ApiAlias::new(
    "azure-openai",
    ApiFormat::OpenAI,
    "https://api.azure.com/v1",
    vec!["gpt-4".to_string()],
    Some("azure/".to_string()),
    false,
    db_service.now(),
  );
  db_service
    .create_api_model_alias(&prefixed_alias, Some("test-key".to_string()))
    .await?;

  // Searching for "gpt-4" should NOT match the prefixed API
  let found = data_service.find_alias("gpt-4").await;
  assert!(
    found.is_none(),
    "Should not match 'gpt-4' when API has prefix 'azure/'"
  );

  // Searching for "azure/gpt-4" SHOULD match
  let found = data_service.find_alias("azure/gpt-4").await;
  assert!(matches!(found, Some(Alias::Api(api)) if api.id == "azure-openai"));

  Ok(())
}
