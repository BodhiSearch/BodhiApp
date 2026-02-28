use crate::{
  access_request_service::{AccessRequestService, DefaultAccessRequestService},
  auth_service::MockAuthService,
  db::{AccessRequestRepository, AppAccessRequestRow, AppAccessRequestStatus, FlowType},
  test_utils::{test_db_service, TestDbService},
};
use anyhow_trace::anyhow_trace;
use objs::{AppError, ApprovalStatus, UserScope};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_draft_popup_valid(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service
    .create_draft(
      "app-client-1".to_string(),
      FlowType::Popup,
      None,
      vec![objs::ToolsetTypeRequest {
        toolset_type: "builtin-exa-search".to_string(),
      }],
      vec![],
      UserScope::User,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Draft, result.status);
  assert_eq!("app-client-1", result.app_client_id);
  assert_eq!(FlowType::Popup, result.flow_type);
  assert_eq!(None, result.redirect_uri);
  assert_eq!("scope_user_user", result.requested_role);
  assert_eq!(None, result.approved_role);
  assert_eq!(None, result.user_id);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_draft_redirect_valid(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service
    .create_draft(
      "app-client-2".to_string(),
      FlowType::Redirect,
      Some("https://example.com/callback".to_string()),
      vec![],
      vec![],
      UserScope::PowerUser,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Draft, result.status);
  assert_eq!(FlowType::Redirect, result.flow_type);
  assert_eq!("scope_user_power_user", result.requested_role);
  // redirect_uri should have ?id=<uuid> appended
  let redirect = result.redirect_uri.unwrap();
  assert!(redirect.starts_with("https://example.com/callback?id="));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_draft_redirect_missing_uri(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service
    .create_draft(
      "app-client-1".to_string(),
      FlowType::Redirect,
      None,
      vec![],
      vec![],
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-missing_redirect_uri", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_already_processed(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let now = db.now();
  let expires_at = now + chrono::Duration::minutes(10);

  // Insert a row that is already approved
  let row = AppAccessRequestRow {
    id: "ar-approved-001".to_string(),
    app_client_id: "app-client-1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"toolset_types":[],"mcp_servers":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[],"mcps":[]}"#.to_string()),
    user_id: Some("user-1".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some("scope_access_request:ar-approved-001".to_string()),
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
  };
  db.create(&row).await?;

  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service
    .approve_request(
      "ar-approved-001",
      "user-2",
      "fake-token",
      vec![],
      vec![],
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-already_processed", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_threads_approved_role(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let now = db.now();
  let expires_at = now + chrono::Duration::minutes(10);

  // Insert a draft row
  let row = AppAccessRequestRow {
    id: "ar-draft-approve".to_string(),
    app_client_id: "app-client-1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"toolset_types":[],"mcp_servers":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
  };
  db.create(&row).await?;

  let db = Arc::new(db);

  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_register_access_request_consent()
    .returning(|_token, _client_id, id, _desc| {
      Ok(crate::auth_service::RegisterAccessRequestConsentResponse {
        access_request_id: id.to_string(),
        access_request_scope: format!("scope_access_request:{}", id),
      })
    });
  let mock_auth = Arc::new(mock_auth);
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service
    .approve_request(
      "ar-draft-approve",
      "user-1",
      "fake-token",
      vec![objs::ToolsetApproval {
        toolset_type: "builtin-exa-search".to_string(),
        status: ApprovalStatus::Approved,
        instance: None,
      }],
      vec![],
      UserScope::PowerUser,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Approved, result.status);
  assert_eq!(Some("user-1".to_string()), result.user_id);
  assert_eq!(
    Some("scope_user_power_user".to_string()),
    result.approved_role
  );
  assert_eq!(
    Some("scope_access_request:ar-draft-approve".to_string()),
    result.access_request_scope
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_request_expired_draft_returns_expired_record(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let now = db.now();
  // Set expires_at to 5 minutes in the past
  let expires_at = now - chrono::Duration::minutes(5);

  let row = AppAccessRequestRow {
    id: "ar-expired-001".to_string(),
    app_client_id: "app-client-1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"toolset_types":[],"mcp_servers":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
  };
  db.create(&row).await?;

  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = service.get_request("ar-expired-001").await?;
  assert!(result.is_some());
  let record = result.unwrap();
  assert_eq!(AppAccessRequestStatus::Expired, record.status);

  Ok(())
}

#[rstest]
#[case::approve("ar-expired-approve")]
#[case::deny("ar-expired-deny")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_request_rejects_expired_draft(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
  #[case] request_id: &str,
) -> anyhow::Result<()> {
  let now = db.now();
  let expires_at = now - chrono::Duration::minutes(5);

  let row = AppAccessRequestRow {
    id: request_id.to_string(),
    app_client_id: "app-client-1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"toolset_types":[],"mcp_servers":[]}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
  };
  db.create(&row).await?;

  let db = Arc::new(db);
  let mock_auth = Arc::new(MockAuthService::new());
  let time_service = Arc::new(crate::test_utils::FrozenTimeService::default());

  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn crate::db::DbService>,
    mock_auth,
    time_service.clone(),
    "http://localhost:1135".to_string(),
  );

  let result = if request_id.contains("approve") {
    service
      .approve_request(
        request_id,
        "user-1",
        "fake-token",
        vec![],
        vec![],
        UserScope::User,
      )
      .await
  } else {
    service.deny_request(request_id, "user-1").await
  };

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-expired", err.code());

  Ok(())
}
