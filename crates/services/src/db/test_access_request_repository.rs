use crate::{
  db::{AccessRequestRepository, AppAccessRequestRow},
  test_utils::{test_db_service, TestDbService},
};
use anyhow_trace::anyhow_trace;
use objs::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_draft_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"toolset_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  let result = service.create(&row).await?;

  assert_eq!(result.id, row.id);
  assert_eq!(result.status, "draft");
  assert_eq!(result.app_client_id, row.app_client_id);
  assert_eq!(result.flow_type, "redirect");
  assert_eq!(
    result.redirect_uri,
    Some("https://example.com/callback".to_string())
  );
  assert_eq!(
    result.requested,
    r#"[{"toolset_type":"builtin-exa-search"}]"#
  );
  assert_eq!(result.approved, None);
  assert_eq!(result.user_id, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "popup".to_string(),
    redirect_uri: None,
    status: "draft".to_string(),
    requested: r#"[{"toolset_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let result = service.get(&row.id).await?;
  assert!(result.is_some());

  let retrieved = result.unwrap();
  assert_eq!(retrieved.id, row.id);
  assert_eq!(retrieved.status, "draft");
  assert_eq!(retrieved.flow_type, "popup");
  assert_eq!(retrieved.redirect_uri, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_nonexistent_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service.get("nonexistent-id").await?;
  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_approval(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440002".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"toolset_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let tools_approved_json =
    r#"[{"toolset_type":"builtin-exa-search","status":"approved","toolset_id":"uuid1"}]"#;
  let result = service
    .update_approval(
      &row.id,
      "user-uuid",
      tools_approved_json,
      "scope_user_user",
      "scope_access_request:550e8400-e29b-41d4-a716-446655440002",
    )
    .await?;

  assert_eq!(result.status, "approved");
  assert_eq!(result.user_id, Some("user-uuid".to_string()));
  assert_eq!(result.approved, Some(tools_approved_json.to_string()));
  assert_eq!(result.approved_role, Some("scope_user_user".to_string()));
  assert_eq!(
    result.access_request_scope,
    Some("scope_access_request:550e8400-e29b-41d4-a716-446655440002".to_string())
  );

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_denial(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440003".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"toolset_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let result = service.update_denial(&row.id, "user-uuid").await?;

  assert_eq!(result.status, "denied");
  assert_eq!(result.user_id, Some("user-uuid".to_string()));
  assert_eq!(result.approved, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_failure(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440004".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"toolset_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let error_msg = "KC registration failed: UUID collision (409).";
  let result = service.update_failure(&row.id, error_msg).await?;

  assert_eq!(result.status, "failed");
  assert_eq!(result.error_message, Some(error_msg.to_string()));

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_by_access_request_scope_found(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::hours(1);

  let scope = "scope_access_request:test-uuid";
  let row = AppAccessRequestRow {
    id: "ar-test-001".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  service.create(&row).await?;

  let result = service.get_by_access_request_scope(scope).await?;

  assert!(result.is_some());
  let found = result.unwrap();
  assert_eq!(found.id, row.id);
  assert_eq!(found.access_request_scope.as_deref(), Some(scope));
  assert_eq!(found.status, "approved");

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_by_access_request_scope_not_found(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service
    .get_by_access_request_scope("scope_access_request:nonexistent")
    .await?;

  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_by_access_request_scope_null(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::hours(1);

  let row = AppAccessRequestRow {
    id: "ar-null-001".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  service.create(&row).await?;

  let result = service
    .get_by_access_request_scope("scope_access_request:anything")
    .await?;

  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_access_request_scope_unique_constraint(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::hours(1);

  let scope = "scope_access_request:duplicate-test";

  let row1 = AppAccessRequestRow {
    id: "ar-dup-1".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  service.create(&row1).await?;

  let row2 = AppAccessRequestRow {
    id: "ar-dup-2".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  let result = service.create(&row2).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code(), "sqlx_error");

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_access_request_scope_multiple_nulls_allowed(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::hours(1);

  let row1 = AppAccessRequestRow {
    id: "ar-null-1".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  service.create(&row1).await?;

  let row2 = AppAccessRequestRow {
    id: "ar-null-2".to_string(),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: Some("Test Description".to_string()),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"exa-001"}}]}"#
        .to_string(),
    ),
    user_id: Some("user-001".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  let result = service.create(&row2).await;

  assert!(result.is_ok());

  Ok(())
}
