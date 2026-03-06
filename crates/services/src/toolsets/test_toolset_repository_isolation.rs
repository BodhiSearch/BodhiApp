use crate::{
  test_utils::{
    sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
  },
  toolsets::{ToolsetEntity, ToolsetRepository},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_toolset(
  id: &str,
  tenant_id: &str,
  user_id: &str,
  slug: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> ToolsetEntity {
  ToolsetEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    user_id: user_id.to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    slug: slug.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: None,
    salt: None,
    nonce: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_toolset_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id_a = ulid::Ulid::new().to_string();
  let id_b = ulid::Ulid::new().to_string();

  let row_a = make_toolset(&id_a, TEST_TENANT_ID, TEST_USER_ID, "toolset-a", ctx.now);
  let row_b = make_toolset(&id_b, TEST_TENANT_B_ID, TEST_USER_ID, "toolset-b", ctx.now);

  ctx.service.create_toolset(TEST_TENANT_ID, &row_a).await?;
  ctx.service.create_toolset(TEST_TENANT_B_ID, &row_b).await?;

  let list_a = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());

  let list_b = ctx
    .service
    .list_toolsets(TEST_TENANT_B_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_b.len());

  let cross = ctx.service.get_toolset(TEST_TENANT_ID, &row_b.id).await?;
  assert_eq!(None, cross);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_toolset_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id_a = ulid::Ulid::new().to_string();
  let id_b = ulid::Ulid::new().to_string();

  let row_a = make_toolset(
    &id_a,
    TEST_TENANT_ID,
    TEST_USER_ID,
    "toolset-user-a",
    ctx.now,
  );
  let row_b = make_toolset(
    &id_b,
    TEST_TENANT_ID,
    TEST_TENANT_A_USER_B_ID,
    "toolset-user-b",
    ctx.now,
  );

  ctx.service.create_toolset(TEST_TENANT_ID, &row_a).await?;
  ctx.service.create_toolset(TEST_TENANT_ID, &row_b).await?;

  let list_a = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());
  assert_eq!("toolset-user-a", list_a[0].slug);

  let list_b = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID)
    .await?;
  assert_eq!(1, list_b.len());
  assert_eq!("toolset-user-b", list_b[0].slug);

  Ok(())
}
