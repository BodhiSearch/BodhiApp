use crate::{
  db::UserAliasRepository,
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use objs::{JsonVec, OAIRequestParams, OAIRequestParamsBuilder, Repo, UserAlias};
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
async fn test_create_and_get_user_alias_by_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let alias = make_alias(&id, "test:model", ctx.now);

  ctx.service.create_user_alias(&alias).await?;

  let fetched = ctx.service.get_user_alias_by_id(&id).await?;
  assert!(fetched.is_some());
  assert_eq!(alias, fetched.unwrap());

  let not_found = ctx.service.get_user_alias_by_id("nonexistent").await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_user_alias_by_name(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let alias = make_alias(&id, "lookup:model", ctx.now);

  ctx.service.create_user_alias(&alias).await?;

  let fetched = ctx.service.get_user_alias_by_name("lookup:model").await?;
  assert!(fetched.is_some());
  assert_eq!(alias, fetched.unwrap());

  let not_found = ctx.service.get_user_alias_by_name("nonexistent").await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_user_alias(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let alias = make_alias(&id, "update:model", ctx.now);

  ctx.service.create_user_alias(&alias).await?;

  let mut updated = alias.clone();
  updated.filename = "updated-model.gguf".to_string();
  updated.snapshot = "v2".to_string();
  updated.request_params = OAIRequestParamsBuilder::default()
    .temperature(0.7f32)
    .build()
    .unwrap();
  updated.context_params = vec!["--ctx-size".to_string(), "4096".to_string()].into();
  ctx.service.update_user_alias(&id, &updated).await?;

  let fetched = ctx
    .service
    .get_user_alias_by_id(&id)
    .await?
    .expect("alias should exist");
  assert_eq!("updated-model.gguf", fetched.filename);
  assert_eq!("v2", fetched.snapshot);
  assert_eq!(Some(0.7), fetched.request_params.temperature);
  assert_eq!(
    vec!["--ctx-size".to_string(), "4096".to_string()],
    *fetched.context_params
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_delete_user_alias(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let alias = make_alias(&id, "delete:model", ctx.now);

  ctx.service.create_user_alias(&alias).await?;
  assert!(ctx.service.get_user_alias_by_id(&id).await?.is_some());

  ctx.service.delete_user_alias(&id).await?;
  assert!(ctx.service.get_user_alias_by_id(&id).await?.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_list_user_aliases_ordered(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let a2 = make_alias(&ulid::Ulid::new().to_string(), "b:model", ctx.now);
  let a1 = make_alias(&ulid::Ulid::new().to_string(), "a:model", ctx.now);

  ctx.service.create_user_alias(&a2).await?;
  ctx.service.create_user_alias(&a1).await?;

  let list = ctx.service.list_user_aliases().await?;
  assert_eq!(2, list.len());
  assert_eq!("a:model", list[0].alias);
  assert_eq!("b:model", list[1].alias);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_user_alias_with_json_fields(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let alias = UserAlias {
    id: id.clone(),
    alias: "json:model".to_string(),
    repo: Repo::try_from("owner/model").unwrap(),
    filename: "model.Q4_K_M.gguf".to_string(),
    snapshot: "abc123".to_string(),
    request_params: OAIRequestParamsBuilder::default()
      .temperature(0.8f32)
      .top_p(0.95f32)
      .max_tokens(2048u32)
      .build()
      .unwrap(),
    context_params: vec!["--ctx-size".to_string(), "2048".to_string()].into(),
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx.service.create_user_alias(&alias).await?;

  let fetched = ctx
    .service
    .get_user_alias_by_id(&id)
    .await?
    .expect("alias should exist");
  assert_eq!(Some(0.8), fetched.request_params.temperature);
  assert_eq!(Some(0.95), fetched.request_params.top_p);
  assert_eq!(Some(2048), fetched.request_params.max_tokens);
  assert_eq!(
    vec!["--ctx-size".to_string(), "2048".to_string()],
    *fetched.context_params
  );
  assert_eq!("owner/model", fetched.repo.to_string());

  Ok(())
}
