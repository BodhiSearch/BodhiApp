use crate::{
  app_access_requests::{AppAccessRequest, AppAccessRequestStatus},
  FlowType,
};
use chrono::Duration;

/// A Draft `AppAccessRequest` row owned by `tenant_id`.
pub(crate) fn make_request(
  id: &str,
  tenant_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> AppAccessRequest {
  AppAccessRequest {
    id: id.to_string(),
    tenant_id: Some(tenant_id.to_string()),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"version":"1"}"#.to_string(),
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

/// An Approved request owned by `user_id` (the shape `update_revocation` /
/// `list_approved_for_user` operate on).
pub(crate) fn approved_request(
  id: &str,
  tenant_id: &str,
  user_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> AppAccessRequest {
  AppAccessRequest {
    status: AppAccessRequestStatus::Approved,
    user_id: Some(user_id.to_string()),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    approved_role: Some("scope_user_user".to_string()),
    ..make_request(id, tenant_id, now)
  }
}
