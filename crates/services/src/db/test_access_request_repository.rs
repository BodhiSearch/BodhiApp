use crate::{
  db::{AccessRequestRepository, AppAccessRequestRow, AppAccessRequestStatus, FlowType},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use chrono::Duration;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_request(id: &str, now: chrono::DateTime<chrono::Utc>) -> AppAccessRequestRow {
  AppAccessRequestRow {
    id: id.to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("A test application".to_string()),
    flow_type: FlowType::Redirect,
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: now + Duration::hours(1),
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_create_and_get_access_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);

  let created = ctx.service.create(&row).await?;
  assert_eq!(row, created);

  let fetched = ctx.service.get(&id).await?;
  assert!(fetched.is_some());
  assert_eq!(row, fetched.unwrap());

  let not_found = ctx.service.get("nonexistent").await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_create_access_request_popup_flow(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let mut row = make_request(&id, ctx.now);
  row.flow_type = FlowType::Popup;
  row.redirect_uri = None;

  let created = ctx.service.create(&row).await?;
  assert_eq!(FlowType::Popup, created.flow_type);
  assert!(created.redirect_uri.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_update_approval(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;

  let approved_json = r#"{"toolsets":[],"mcps":[]}"#;
  let updated = ctx
    .service
    .update_approval(
      &id,
      "approver-user",
      approved_json,
      "scope_user_user",
      &format!("scope_access_request:{}", id),
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Approved, updated.status);
  assert_eq!(Some("approver-user".to_string()), updated.user_id);
  assert_eq!(Some(approved_json.to_string()), updated.approved);
  assert_eq!(Some("scope_user_user".to_string()), updated.approved_role);
  assert_eq!(
    Some(format!("scope_access_request:{}", id)),
    updated.access_request_scope
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_update_denial(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;

  let updated = ctx.service.update_denial(&id, "denier-user").await?;

  assert_eq!(AppAccessRequestStatus::Denied, updated.status);
  assert_eq!(Some("denier-user".to_string()), updated.user_id);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_update_failure(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;

  let updated = ctx
    .service
    .update_failure(&id, "token exchange failed")
    .await?;

  assert_eq!(AppAccessRequestStatus::Failed, updated.status);
  assert_eq!(
    Some("token exchange failed".to_string()),
    updated.error_message
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_get_by_access_request_scope(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;

  let scope = format!("scope_access_request:{}", id);
  ctx
    .service
    .update_approval(&id, "user-1", "{}", "scope_user_user", &scope)
    .await?;

  let found = ctx.service.get_by_access_request_scope(&scope).await?;
  assert!(found.is_some());
  assert_eq!(id, found.unwrap().id);

  let not_found = ctx
    .service
    .get_by_access_request_scope("nonexistent-scope")
    .await?;
  assert!(not_found.is_none());

  Ok(())
}
