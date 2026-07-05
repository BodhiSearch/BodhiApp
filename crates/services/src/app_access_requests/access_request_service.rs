use async_trait::async_trait;
use chrono::Duration;
use std::sync::Arc;

use super::error::{AccessRequestError, Result};
use super::{
  AppAccessRequest, AppAccessRequestStatus, ApprovalStatus, ApprovedResources, RequestedResources,
};
use crate::db::{DbService, TimeService};
use crate::new_ulid;
use crate::AuthService;
use crate::UserScope;

/// App access request lifecycle service.
///
/// NOTE: This service is intentionally NOT auth-scoped. Unlike other domain services where
/// tenant_id/user_id scope which records are visible, app access requests have a different
/// lifecycle:
/// - create_draft: Anonymous (no authenticated user), tenant_id is NULL
/// - get_request: Used by both anonymous status polling and authenticated review
/// - approve/deny: reviewer's user_id is recorded as actor, tenant_id bound at approval
///
/// Exposed on AuthScopedAppService as a non-auth-scoped passthrough for convenience.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AccessRequestService: Send + Sync + std::fmt::Debug {
  /// tenant_id is NULL — bound at approval time. `source_access_request_id` is the
  /// prior request an upgrade elevates (handler resolves it from the caller's token).
  async fn create_draft(
    &self,
    app_client_id: String,
    requested: RequestedResources,
    requested_role: UserScope,
    source_access_request_id: Option<String>,
  ) -> Result<AppAccessRequest>;

  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequest>>;

  /// `tenant_id` binds the draft to the approver's active tenant.
  #[allow(clippy::too_many_arguments)]
  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    tenant_id: &str,
    user_token: &str,
    approved: ApprovedResources,
    approved_role: UserScope,
  ) -> Result<AppAccessRequest>;

  async fn deny_request(&self, id: &str, user_id: &str) -> Result<AppAccessRequest>;

  /// Approved access requests (issued app tokens) owned by `user_id` in `tenant_id`.
  async fn list_approved_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<AppAccessRequest>>;

  /// Revoke a previously-approved grant owned by `user_id`; the app token stops working.
  async fn revoke_request(
    &self,
    tenant_id: &str,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequest>;

  fn build_review_url(&self, access_request_id: &str) -> String;

  /// Canonical authorize endpoint the review page validates the app-supplied `auth_url` against.
  fn build_authorize_endpoint(&self) -> String;
}

#[derive(Debug)]
pub struct DefaultAccessRequestService {
  db_service: Arc<dyn DbService>,
  auth_service: Arc<dyn AuthService>,
  time_service: Arc<dyn TimeService>,
  frontend_url: String,
}

impl DefaultAccessRequestService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    auth_service: Arc<dyn AuthService>,
    time_service: Arc<dyn TimeService>,
    frontend_url: String,
  ) -> Self {
    Self {
      db_service,
      auth_service,
      time_service,
      frontend_url,
    }
  }

  fn generate_description(&self, approved: &ApprovedResources) -> String {
    let mut lines = Vec::new();
    match approved {
      ApprovedResources::V1(v1) => {
        for approval in &v1.mcps {
          if approval.status == ApprovalStatus::Approved {
            lines.push(format!("- MCP: {}", approval.url));
          }
        }
      }
    }
    if lines.is_empty() {
      "Access approved".to_string()
    } else {
      lines.join("\n")
    }
  }
}

#[async_trait]
impl AccessRequestService for DefaultAccessRequestService {
  async fn create_draft(
    &self,
    app_client_id: String,
    requested: RequestedResources,
    requested_role: UserScope,
    source_access_request_id: Option<String>,
  ) -> Result<AppAccessRequest> {
    let access_request_id = new_ulid();

    let now = self.time_service.utc_now();
    let expires_at = now + Duration::minutes(10);

    let requested_json = serde_json::to_string(&requested).map_err(|e| {
      AccessRequestError::InvalidStatus(format!("JSON serialization failed: {}", e))
    })?;

    let row = AppAccessRequest {
      id: access_request_id,
      tenant_id: None,
      app_client_id,
      app_name: None,
      app_description: None,
      status: AppAccessRequestStatus::Draft,
      requested: requested_json,
      approved: None,
      user_id: None,
      requested_role: requested_role.to_string(),
      approved_role: None,
      access_request_scope: None,
      source_access_request_id,
      error_message: None,
      expires_at,
      created_at: now,
      updated_at: now,
    };

    let created_row = self.db_service.create(&row).await?;
    Ok(created_row)
  }

  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequest>> {
    let row = self.db_service.get("", id).await?;
    Ok(row)
  }

  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    tenant_id: &str,
    user_token: &str,
    approved: ApprovedResources,
    approved_role: UserScope,
  ) -> Result<AppAccessRequest> {
    let row = self
      .get_request(id)
      .await?
      .ok_or_else(|| AccessRequestError::NotFound(id.to_string()))?;

    match row.status {
      AppAccessRequestStatus::Draft => {}
      AppAccessRequestStatus::Expired => {
        return Err(AccessRequestError::Expired(id.to_string()));
      }
      _ => {
        return Err(AccessRequestError::AlreadyProcessed(id.to_string()));
      }
    }

    let requested_resources: RequestedResources = serde_json::from_str(&row.requested)
      .map_err(|e| AccessRequestError::InvalidStatus(format!("Invalid requested JSON: {}", e)))?;
    if requested_resources.version() != approved.version() {
      return Err(AccessRequestError::VersionMismatch {
        requested_version: requested_resources.version().to_string(),
        approved_version: approved.version().to_string(),
      });
    }

    let description = self.generate_description(&approved);

    let kc_response = match self
      .auth_service
      .register_access_request_consent(user_token, &row.app_client_id, id, &description)
      .await
    {
      Ok(resp) => resp,
      Err(e) => {
        let error_msg = e.to_string();
        if error_msg.contains("409") || error_msg.contains("UUID collision") {
          let failure_msg =
            "KC registration failed: UUID collision (409). Please retry with new request.";
          let failed_row = self.db_service.update_failure(id, failure_msg).await?;
          return Ok(failed_row);
        } else {
          return Err(AccessRequestError::KcRegistrationFailed(error_msg));
        }
      }
    };

    let approved_json = serde_json::to_string(&approved).map_err(|e| {
      AccessRequestError::InvalidStatus(format!("JSON serialization failed: {}", e))
    })?;

    let updated_row = self
      .db_service
      .update_approval(
        id,
        user_id,
        tenant_id,
        &approved_json,
        &approved_role.to_string(),
        &kc_response.access_request_scope,
      )
      .await?;

    Ok(updated_row)
  }

  async fn deny_request(&self, id: &str, user_id: &str) -> Result<AppAccessRequest> {
    let row = self
      .get_request(id)
      .await?
      .ok_or_else(|| AccessRequestError::NotFound(id.to_string()))?;

    match row.status {
      AppAccessRequestStatus::Draft => {}
      AppAccessRequestStatus::Expired => {
        return Err(AccessRequestError::Expired(id.to_string()));
      }
      _ => {
        return Err(AccessRequestError::AlreadyProcessed(id.to_string()));
      }
    }

    let updated_row = self.db_service.update_denial(id, user_id).await?;
    Ok(updated_row)
  }

  async fn list_approved_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<AppAccessRequest>> {
    let rows = self
      .db_service
      .list_approved_for_user(tenant_id, user_id)
      .await?;
    Ok(rows)
  }

  async fn revoke_request(
    &self,
    tenant_id: &str,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequest> {
    // The token-exchange path requires status == Approved, so flipping to Revoked
    // stops the app token. Keycloak consent is left in place (best-effort); a
    // revoked record fails exchange regardless.
    let updated_row = self
      .db_service
      .update_revocation(tenant_id, id, user_id)
      .await?;
    Ok(updated_row)
  }

  fn build_review_url(&self, access_request_id: &str) -> String {
    format!(
      "{}/ui/apps/access-requests/review?id={}",
      self.frontend_url, access_request_id
    )
  }

  fn build_authorize_endpoint(&self) -> String {
    self.auth_service.authorize_url()
  }
}

#[cfg(test)]
#[path = "test_access_request_service.rs"]
mod test_access_request_service;
