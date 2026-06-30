use serde::{Deserialize, Serialize};
use services::UserInfo;
use services::{
  ApprovalStatus, ApprovedResourcesV1, McpGrant, ModelGrant, TokenGrantsV1, TokenScope,
};
use utoipa::ToSchema;

/// Token Type
/// `session` - token stored in cookie based http session
/// `bearer` - token received from http authorization header as bearer token
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
  Session,
  Bearer,
}

/// Role Source
/// `role` - client level user role
/// `scope_token` - scope granted token role
/// `scope_user` - scope granted user role
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleSource {
  Role,
  ScopeToken,
  ScopeUser,
}

/// Effective access to a class of resources (models or MCPs) for an API token,
/// reflected from its grants. Discriminated on `type`: `all` ⇒ every current and
/// future resource; `specific` ⇒ the listed `ids` (empty ⇒ no access).
/// `list` is the `list_*` toggle (whether the token may enumerate the full catalog).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResourceAccess {
  All { list: bool },
  Specific { list: bool, ids: Vec<String> },
}

impl ResourceAccess {
  pub fn models(grants: &TokenGrantsV1) -> Self {
    match &grants.models {
      ModelGrant::All => Self::All {
        list: grants.models_list,
      },
      ModelGrant::Specific { ids } => Self::Specific {
        list: grants.models_list,
        ids: ids.clone(),
      },
    }
  }

  pub fn mcps(grants: &TokenGrantsV1) -> Self {
    match &grants.mcps {
      McpGrant::All => Self::All {
        list: grants.mcps_list,
      },
      McpGrant::Specific { ids } => Self::Specific {
        list: grants.mcps_list,
        ids: ids.clone(),
      },
    }
  }

  /// Effective model access reflected from an approved app grant.
  pub fn app_models(grants: &ApprovedResourcesV1) -> Self {
    match &grants.models_access {
      ModelGrant::All => Self::All {
        list: grants.models_list,
      },
      ModelGrant::Specific { ids } => Self::Specific {
        list: grants.models_list,
        ids: ids.clone(),
      },
    }
  }

  /// Effective MCP access reflected from an approved app grant: the union of the
  /// by-url approved instances and the owner-extra grant.
  pub fn app_mcps(grants: &ApprovedResourcesV1) -> Self {
    match &grants.mcps_access {
      McpGrant::All => Self::All {
        list: grants.mcps_list,
      },
      McpGrant::Specific { ids } => {
        let mut all_ids: Vec<String> = grants
          .mcps
          .iter()
          .filter(|a| a.status == ApprovalStatus::Approved)
          .filter_map(|a| a.instance.as_ref().map(|i| i.id.clone()))
          .collect();
        for id in ids {
          if !all_ids.contains(id) {
            all_ids.push(id.clone());
          }
        }
        Self::Specific {
          list: grants.mcps_list,
          ids: all_ids,
        }
      }
    }
  }
}

/// Effective resource access for a token-bearing principal (API token or external
/// app), reflected from its grants. Reported uniformly via the `access` envelope
/// field for both principals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ResourceAccessInfo {
  /// Effective model access for this principal.
  pub models: ResourceAccess,
  /// Effective MCP access for this principal.
  pub mcps: ResourceAccess,
}

/// API Token information response. Effective model/MCP access is reported uniformly
/// via the envelope's `access` field (same shape as external apps), not inline here.
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct TokenInfo {
  pub role: TokenScope,
}

/// User authentication response with discriminated union
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "auth_status")]
#[schema(example = json!({
    "auth_status": "logged_in",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "user@example.com",
    "role": "resource_user"
}))]
pub enum UserResponse {
  /// User is not authenticated
  #[serde(rename = "logged_out")]
  LoggedOut,
  /// User is authenticated with details
  #[serde(rename = "logged_in")]
  LoggedIn(UserInfo),
  /// API token authentication
  #[serde(rename = "api_token")]
  Token(TokenInfo),
}

/// Dashboard user information from a validated dashboard session token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DashboardUser {
  pub user_id: String,
  pub username: String,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
}

/// Envelope wrapping UserResponse with additional session info
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserInfoEnvelope {
  /// Core user authentication response
  #[serde(flatten)]
  pub user: UserResponse,
  /// Dashboard user info when a validated dashboard session exists
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub dashboard: Option<DashboardUser>,
  /// Effective resource access — present for token-bearing principals (API token
  /// or external app); absent for sessions and anonymous.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub access: Option<ResourceAccessInfo>,
}

/// List users query parameters. Intentionally omits sort fields (unlike PaginationSortParams)
/// because user listing is fetched from the auth service which handles its own ordering.
#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct ListUsersParams {
  #[schema(example = 1)]
  pub page: Option<u32>,
  #[schema(example = 10)]
  pub page_size: Option<u32>,
}
