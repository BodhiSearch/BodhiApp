use async_trait::async_trait;
use chrono::Duration;
use std::sync::Arc;
use ulid::Ulid;

use crate::access_request_service::error::{AccessRequestError, Result};
use crate::auth_service::AuthService;
use crate::db::{AppAccessRequestRow, AppAccessRequestStatus, DbService, FlowType, TimeService};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AccessRequestService: Send + Sync + std::fmt::Debug {
  /// Create a draft access request
  async fn create_draft(
    &self,
    app_client_id: String,
    flow_type: String,
    redirect_uri: Option<String>,
    tools_requested: Vec<objs::ToolsetTypeRequest>,
    mcp_servers_requested: Vec<objs::McpServerRequest>,
    requested_role: String,
  ) -> Result<AppAccessRequestRow>;

  /// Get access request by ID
  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>>;

  /// Approve access request and register with KC
  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    user_token: &str,
    tool_approvals: Vec<objs::ToolsetApproval>,
    mcp_approvals: Vec<objs::McpApproval>,
    approved_role: String,
  ) -> Result<AppAccessRequestRow>;

  /// Deny access request
  async fn deny_request(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow>;

  /// Build review URL for a given access request ID
  fn build_review_url(&self, access_request_id: &str) -> String;
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

  fn generate_description(
    &self,
    tool_approvals: &[objs::ToolsetApproval],
    mcp_approvals: &[objs::McpApproval],
  ) -> String {
    let mut lines = Vec::new();
    for approval in tool_approvals {
      if approval.status == "approved" {
        lines.push(format!("- {}", approval.toolset_type));
      }
    }
    for approval in mcp_approvals {
      if approval.status == "approved" {
        lines.push(format!("- MCP: {}", approval.url));
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
    flow_type: String,
    redirect_uri: Option<String>,
    toolsets_requested: Vec<objs::ToolsetTypeRequest>,
    mcp_servers_requested: Vec<objs::McpServerRequest>,
    requested_role: String,
  ) -> Result<AppAccessRequestRow> {
    let flow_type_enum = match flow_type.as_str() {
      "redirect" => FlowType::Redirect,
      "popup" => FlowType::Popup,
      _ => return Err(AccessRequestError::InvalidFlowType(flow_type)),
    };

    if flow_type_enum == FlowType::Redirect && redirect_uri.is_none() {
      return Err(AccessRequestError::MissingRedirectUri);
    }

    let access_request_id = Ulid::new().to_string();

    let now = self.time_service.utc_now();
    let expires_at = now + Duration::minutes(10);

    let requested_json = serde_json::to_string(&serde_json::json!({
      "toolset_types": toolsets_requested,
      "mcp_servers": mcp_servers_requested,
    }))
    .map_err(|e| AccessRequestError::InvalidStatus(format!("JSON serialization failed: {}", e)))?;

    let modified_redirect_uri = redirect_uri.map(|uri| {
      if uri.contains('?') {
        format!("{}&id={}", uri, access_request_id)
      } else {
        format!("{}?id={}", uri, access_request_id)
      }
    });

    let row = AppAccessRequestRow {
      id: access_request_id,
      app_client_id,
      app_name: None,
      app_description: None,
      flow_type: flow_type_enum,
      redirect_uri: modified_redirect_uri,
      status: AppAccessRequestStatus::Draft,
      requested: requested_json,
      approved: None,
      user_id: None,
      requested_role,
      approved_role: None,
      access_request_scope: None,
      error_message: None,
      expires_at,
      created_at: now,
      updated_at: now,
    };

    let created_row = self.db_service.create(&row).await?;
    Ok(created_row)
  }

  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>> {
    let row = self.db_service.get(id).await?;

    if let Some(row) = row {
      // Check expiry for draft status
      if row.status == AppAccessRequestStatus::Draft {
        let now = self.time_service.utc_now();
        if row.expires_at < now {
          return Err(AccessRequestError::Expired(id.to_string()));
        }
      }
      Ok(Some(row))
    } else {
      Ok(None)
    }
  }

  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    user_token: &str,
    tool_approvals: Vec<objs::ToolsetApproval>,
    mcp_approvals: Vec<objs::McpApproval>,
    approved_role: String,
  ) -> Result<AppAccessRequestRow> {
    let row = self
      .get_request(id)
      .await?
      .ok_or_else(|| AccessRequestError::NotFound(id.to_string()))?;

    if row.status != AppAccessRequestStatus::Draft {
      return Err(AccessRequestError::AlreadyProcessed(id.to_string()));
    }

    let description = self.generate_description(&tool_approvals, &mcp_approvals);

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

    let approved_json = serde_json::to_string(&serde_json::json!({
      "toolsets": tool_approvals,
      "mcps": mcp_approvals,
    }))
    .map_err(|e| AccessRequestError::InvalidStatus(format!("JSON serialization failed: {}", e)))?;

    let updated_row = self
      .db_service
      .update_approval(
        id,
        user_id,
        &approved_json,
        &approved_role,
        &kc_response.access_request_scope,
      )
      .await?;

    Ok(updated_row)
  }

  async fn deny_request(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow> {
    // Get current request
    let row = self
      .get_request(id)
      .await?
      .ok_or_else(|| AccessRequestError::NotFound(id.to_string()))?;

    if row.status != AppAccessRequestStatus::Draft {
      return Err(AccessRequestError::AlreadyProcessed(id.to_string()));
    }

    let updated_row = self.db_service.update_denial(id, user_id).await?;
    Ok(updated_row)
  }

  fn build_review_url(&self, access_request_id: &str) -> String {
    format!(
      "{}/ui/apps/access-requests/review?id={}",
      self.frontend_url, access_request_id
    )
  }
}

#[cfg(test)]
#[path = "test_access_request_service.rs"]
mod test_access_request_service;
