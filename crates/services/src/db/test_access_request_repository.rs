use crate::{
  db::{AccessRequestRepository, AppAccessRequestRow, AppAccessRequestStatus, DbError, FlowType},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use chrono::Duration;
use objs::AppError;
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
async fn test_create_and_get_access_request(
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
async fn test_create_access_request_popup_flow(
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
async fn test_update_approval(
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
async fn test_update_denial(
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
async fn test_update_failure(
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
async fn test_get_by_access_request_scope(
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

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_marks_expired_draft(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let mut row = make_request(&id, ctx.now);
  // Set expires_at in the past
  row.expires_at = ctx.now - Duration::minutes(5);
  ctx.service.create(&row).await?;

  let fetched = ctx.service.get(&id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(AppAccessRequestStatus::Expired, fetched.status);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_returns_draft_when_not_expired(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;

  let fetched = ctx.service.get(&id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(AppAccessRequestStatus::Draft, fetched.status);

  Ok(())
}

/// Helper enum to parameterize update operations across approval/denial/failure
enum UpdateOp {
  Approval,
  Denial,
  Failure,
}

async fn perform_update(
  service: &crate::db::DefaultDbService,
  id: &str,
  op: &UpdateOp,
) -> Result<AppAccessRequestRow, DbError> {
  match op {
    UpdateOp::Approval => {
      service
        .update_approval(id, "user-1", "{}", "scope_user_user", "scope")
        .await
    }
    UpdateOp::Denial => service.update_denial(id, "user-1").await,
    UpdateOp::Failure => service.update_failure(id, "some error").await,
  }
}

/// Transition a draft to a non-draft state so we can test rejection of updates on non-draft records
async fn transition_to_non_draft(
  service: &crate::db::DefaultDbService,
  id: &str,
  op: &UpdateOp,
) -> Result<(), DbError> {
  match op {
    UpdateOp::Approval => {
      service
        .update_approval(id, "user-1", "{}", "scope_user_user", "scope")
        .await?;
    }
    UpdateOp::Denial => {
      service.update_denial(id, "user-1").await?;
    }
    UpdateOp::Failure => {
      service.update_failure(id, "original error").await?;
    }
  }
  Ok(())
}

#[rstest]
#[case::approval(UpdateOp::Approval)]
#[case::denial(UpdateOp::Denial)]
#[case::failure(UpdateOp::Failure)]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_rejects_expired_draft(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
  #[case] op: UpdateOp,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let mut row = make_request(&id, ctx.now);
  row.expires_at = ctx.now - Duration::minutes(5);
  ctx.service.create(&row).await?;

  let result = perform_update(&ctx.service, &id, &op).await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("db_error-access_request_expired", err.code());

  Ok(())
}

#[rstest]
#[case::approval(UpdateOp::Approval)]
#[case::denial(UpdateOp::Denial)]
#[case::failure(UpdateOp::Failure)]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_rejects_non_draft(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
  #[case] op: UpdateOp,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = ulid::Ulid::new().to_string();
  let row = make_request(&id, ctx.now);
  ctx.service.create(&row).await?;
  transition_to_non_draft(&ctx.service, &id, &op).await?;

  let result = perform_update(&ctx.service, &id, &op).await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("db_error-access_request_not_draft", err.code());

  Ok(())
}
