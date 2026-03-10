use crate::models::{JsonVec, OAIRequestParams, Repo, UserAlias, UserAliasRepository};
use crate::new_ulid;
use crate::test_utils::{
  sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_alias(id: &str, alias: &str, now: chrono::DateTime<chrono::Utc>) -> UserAlias {
  UserAlias {
    id: id.to_string(),
    alias: alias.to_string(),
    repo: Repo::try_from("test/repo").unwrap(),
    filename: "model.gguf".to_string(),
    snapshot: "main".to_string(),
    request_params: OAIRequestParams::default(),
    context_params: JsonVec::default(),
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_user_alias_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_a = make_alias(&new_ulid(), "alias:tenant-a", ctx.now);
  let alias_b = make_alias(&new_ulid(), "alias:tenant-b", ctx.now);

  ctx
    .service
    .create_user_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_a)
    .await?;
  ctx
    .service
    .create_user_alias(TEST_TENANT_B_ID, TEST_USER_ID, &alias_b)
    .await?;

  // Listing aliases in tenant A should only return alias_a
  let list_a = ctx
    .service
    .list_user_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());
  assert_eq!("alias:tenant-a", list_a[0].alias);

  // Listing aliases in tenant B should only return alias_b
  let list_b = ctx
    .service
    .list_user_aliases(TEST_TENANT_B_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_b.len());
  assert_eq!("alias:tenant-b", list_b[0].alias);

  // Getting alias_b by ID under tenant A should return None
  let cross = ctx
    .service
    .get_user_alias_by_id(TEST_TENANT_ID, TEST_USER_ID, &alias_b.id)
    .await?;
  assert_eq!(None, cross);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_alias_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias_user_a = make_alias(&new_ulid(), "alias:user-a", ctx.now);
  let alias_user_b = make_alias(&new_ulid(), "alias:user-b", ctx.now);

  ctx
    .service
    .create_user_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_user_a)
    .await?;
  ctx
    .service
    .create_user_alias(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, &alias_user_b)
    .await?;

  // Listing aliases for user A should only return alias_user_a
  let list_a = ctx
    .service
    .list_user_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());
  assert_eq!("alias:user-a", list_a[0].alias);

  // Listing aliases for user B should only return alias_user_b
  let list_b = ctx
    .service
    .list_user_aliases(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID)
    .await?;
  assert_eq!(1, list_b.len());
  assert_eq!("alias:user-b", list_b[0].alias);

  Ok(())
}
