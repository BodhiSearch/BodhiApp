use async_trait::async_trait;
use std::sync::Arc;

use super::app_scopes::access_request_scope_value;
use super::error::{AccessRequestError, Result};
use super::{AppAccessRequest, AppAccessRequestStatus, CreateApprovedAccessRequest};
use crate::db::{DbService, TimeService};
use crate::new_ulid;
use crate::AuthService;

/// App access request lifecycle service. Rows are created already-approved at consent
/// (the user is authenticated there), with tenant/user bound from the session.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AccessRequestService: Send + Sync + std::fmt::Debug {
  /// Create an Approved row; its `access_request_scope` is the dotted dynamic scope
  /// `scope_access_request:<resource-client-id>.<row-id>` the Keycloak mapper parses.
  async fn create_approved(&self, input: CreateApprovedAccessRequest) -> Result<AppAccessRequest>;

  async fn get_request(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>>;

  /// Newest approved grant for `(tenant, app, user)` — the reauthorize prefill source.
  async fn latest_approved_for_app_user(
    &self,
    tenant_id: &str,
    app_client_id: &str,
    user_id: &str,
  ) -> Result<Option<AppAccessRequest>>;

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

  /// Canonical Keycloak authorize endpoint the consent flow redirects to.
  fn build_authorize_endpoint(&self) -> String;
}

#[derive(Debug)]
pub struct DefaultAccessRequestService {
  db_service: Arc<dyn DbService>,
  auth_service: Arc<dyn AuthService>,
  time_service: Arc<dyn TimeService>,
}

impl DefaultAccessRequestService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    auth_service: Arc<dyn AuthService>,
    time_service: Arc<dyn TimeService>,
  ) -> Self {
    Self {
      db_service,
      auth_service,
      time_service,
    }
  }
}

#[async_trait]
impl AccessRequestService for DefaultAccessRequestService {
  async fn create_approved(&self, input: CreateApprovedAccessRequest) -> Result<AppAccessRequest> {
    let id = new_ulid();
    let access_request_scope = access_request_scope_value(&input.resource_client_id, &id)?;

    let requested_json = serde_json::to_string(&input.requested)
      .map_err(|e| AccessRequestError::Serialization(e.to_string()))?;
    let approved_json = serde_json::to_string(&input.approved)
      .map_err(|e| AccessRequestError::Serialization(e.to_string()))?;

    let now = self.time_service.utc_now();
    let row = AppAccessRequest {
      id,
      tenant_id: Some(input.tenant_id),
      app_client_id: input.app_client_id,
      app_name: None,
      app_description: None,
      status: AppAccessRequestStatus::Approved,
      requested: requested_json,
      approved: Some(approved_json),
      user_id: Some(input.user_id),
      requested_role: input.requested_role.to_string(),
      approved_role: Some(input.approved_role.to_string()),
      access_request_scope: Some(access_request_scope),
      source_access_request_id: input.source_access_request_id,
      error_message: None,
      // Approved rows never expire; the column is NOT NULL so it carries the creation time.
      expires_at: now,
      created_at: now,
      updated_at: now,
    };

    let created_row = self.db_service.create(&row).await?;
    Ok(created_row)
  }

  async fn get_request(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>> {
    let row = self.db_service.get(tenant_id, id).await?;
    Ok(row)
  }

  async fn latest_approved_for_app_user(
    &self,
    tenant_id: &str,
    app_client_id: &str,
    user_id: &str,
  ) -> Result<Option<AppAccessRequest>> {
    let row = self
      .db_service
      .latest_approved_for_app_user(tenant_id, app_client_id, user_id)
      .await?;
    Ok(row)
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
    // stops the app token.
    let updated_row = self
      .db_service
      .update_revocation(tenant_id, id, user_id)
      .await?;
    Ok(updated_row)
  }

  fn build_authorize_endpoint(&self) -> String {
    self.auth_service.authorize_url()
  }
}

#[cfg(test)]
#[path = "test_access_request_service.rs"]
mod test_access_request_service;
