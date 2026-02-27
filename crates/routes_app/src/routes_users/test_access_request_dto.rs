use crate::{ApproveUserAccessRequest, PaginatedUserAccessResponse, UserAccessStatusResponse};
use objs::ResourceRole;
use pretty_assertions::assert_eq;
use serde_json::json;
use services::db::{UserAccessRequest, UserAccessRequestStatus};

#[test]
fn test_user_access_status_response_from_user_access_request() {
  // Test DTO conversion
  let request = UserAccessRequest {
    id: "01JNFG0000000000000000TEST".to_string(),
    username: "test@example.com".to_string(),
    user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    reviewer: None,
    status: UserAccessRequestStatus::Pending,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let response = UserAccessStatusResponse::from(request.clone());

  assert_eq!(request.username, response.username);
  assert_eq!(request.status, response.status);
  assert_eq!(request.created_at, response.created_at);
  assert_eq!(request.updated_at, response.updated_at);
}

#[test]
fn test_approve_user_access_request_serde() -> anyhow::Result<()> {
  // Test request deserialization
  let json = r#"{"role": "resource_user"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json)?;
  assert_eq!(ResourceRole::User, request.role);

  let json = r#"{"role": "resource_admin"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json)?;
  assert_eq!(ResourceRole::Admin, request.role);

  Ok(())
}

#[test]
fn test_paginated_user_access_response_serde() -> anyhow::Result<()> {
  // Test response serialization
  let response = PaginatedUserAccessResponse {
    requests: vec![],
    total: 0,
    page: 1,
    page_size: 20,
  };

  let json: serde_json::Value = serde_json::to_value(&response)?;
  assert_eq!(json!([]), json["requests"]);
  assert_eq!(0, json["total"]);
  assert_eq!(1, json["page"]);
  assert_eq!(20, json["page_size"]);

  Ok(())
}
