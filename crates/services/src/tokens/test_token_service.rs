use super::{DefaultTokenService, TokenService};
use crate::test_utils::{
  fixed_dt, test_db_service, token_entity, FrozenTimeService, TestDbService, TEST_TENANT_ID,
};
use crate::TokenScope;
use crate::TokenStatus;
use crate::{
  token_checksum, CreateTokenRequest, UpdateTokenRequest, BODHIAPP_TOKEN_PREFIX, TOKEN_CHECKSUM_LEN,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use std::sync::Arc;

#[fixture]
#[awt]
async fn token_service(#[future] test_db_service: TestDbService) -> DefaultTokenService {
  let time_service = Arc::new(FrozenTimeService::default());
  DefaultTokenService::new(Arc::new(test_db_service), time_service)
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_and_get_api_token(
  #[future] token_service: DefaultTokenService,
) -> anyhow::Result<()> {
  let mut token = token_entity("tok_test1", "user1", "bt_abc", fixed_dt());
  token.name = "test-token".to_string();

  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let retrieved = token_service
    .get_api_token_by_id(TEST_TENANT_ID, "user1", "tok_test1")
    .await?;
  assert!(retrieved.is_some());
  let retrieved = retrieved.unwrap();
  assert_eq!("test-token", retrieved.name);
  assert_eq!("bt_abc", retrieved.token_prefix);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_api_tokens(#[future] token_service: DefaultTokenService) -> anyhow::Result<()> {
  for i in 0..3 {
    let mut token = token_entity(&format!("tok_{i}"), "user1", &format!("bt_{i}"), fixed_dt());
    token_service
      .create_api_token(TEST_TENANT_ID, &mut token)
      .await?;
  }

  let response = token_service
    .list_api_tokens(TEST_TENANT_ID, "user1", 1, 10)
    .await?;
  assert_eq!(3, response.total);
  assert_eq!(3, response.data.len());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_api_token_by_prefix(
  #[future] token_service: DefaultTokenService,
) -> anyhow::Result<()> {
  let mut token = token_entity("tok_prefix", "user1", "bt_unique", fixed_dt());
  token.name = "prefix-token".to_string();
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let result = token_service.get_api_token_by_prefix("bt_unique").await?;
  assert!(result.is_some());
  assert_eq!("prefix-token", result.unwrap().name);

  let result = token_service.get_api_token_by_prefix("bt_nonexist").await?;
  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_api_token(#[future] token_service: DefaultTokenService) -> anyhow::Result<()> {
  let mut token = token_entity("tok_update", "user1", "bt_upd", fixed_dt());
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  token.name = "updated-name".to_string();
  token.status = TokenStatus::Inactive;
  token_service
    .update_api_token(TEST_TENANT_ID, "user1", &mut token)
    .await?;

  let updated = token_service
    .get_api_token_by_id(TEST_TENANT_ID, "user1", "tok_update")
    .await?
    .unwrap();
  assert_eq!("updated-name", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_generates_valid_token(
  #[future] token_service: DefaultTokenService,
) -> anyhow::Result<()> {
  let form = CreateTokenRequest {
    name: Some("my-token".to_string()),
    scope: TokenScope::User,
    grants: Default::default(),
  };
  let token_created = token_service
    .create_token(TEST_TENANT_ID, "user1", form)
    .await?;

  let raw_token = &token_created.token;

  assert!(raw_token.starts_with(BODHIAPP_TOKEN_PREFIX));

  assert!(
    raw_token.contains('.'),
    "Token should contain '.' separator"
  );

  // <random><checksum> segment before the .<client_id> suffix
  let rand_and_sum = raw_token
    .strip_prefix(BODHIAPP_TOKEN_PREFIX)
    .unwrap()
    .split('.')
    .next()
    .unwrap();
  let split = rand_and_sum.len() - TOKEN_CHECKSUM_LEN;
  let (random_part, checksum) = (&rand_and_sum[..split], &rand_and_sum[split..]);
  assert_eq!(
    token_checksum(random_part),
    checksum,
    "embedded checksum should verify against the random segment"
  );
  let token_prefix = format!("{}{}", BODHIAPP_TOKEN_PREFIX, &random_part[..8]);

  let retrieved = token_service.get_api_token_by_prefix(&token_prefix).await?;
  assert!(retrieved.is_some());
  let entity = retrieved.unwrap();

  assert_eq!("my-token", entity.name);
  assert_eq!("user1", entity.user_id);
  assert_eq!("scope_token_user", entity.scopes);
  assert_eq!(TokenStatus::Active, entity.status);
  assert_eq!(token_prefix, entity.token_prefix);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_token_with_form(
  #[future] token_service: DefaultTokenService,
) -> anyhow::Result<()> {
  let mut token = token_entity("tok_form_update", "user1", "bt_form", fixed_dt());
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let form = UpdateTokenRequest {
    name: "updated-via-form".to_string(),
    status: TokenStatus::Inactive,
  };
  let updated = token_service
    .update_token(TEST_TENANT_ID, "user1", "tok_form_update", form)
    .await?;

  assert_eq!("updated-via-form", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);

  Ok(())
}
