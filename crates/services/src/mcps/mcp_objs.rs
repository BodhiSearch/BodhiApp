use chrono::{DateTime, Utc};
use mcp_client::McpTool;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// McpServer - Admin-managed MCP server registry (public API model)
// ============================================================================

/// Admin-managed MCP server registry entry.
/// Admins/managers register MCP server URLs that users can then create instances of.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServer {
  /// Unique identifier (UUID)
  pub id: String,
  /// MCP server endpoint URL (trimmed, case-insensitive unique)
  pub url: String,
  /// Human-readable display name
  pub name: String,
  /// Optional description
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this MCP server is enabled
  pub enabled: bool,
  /// User who created this entry
  pub created_by: String,
  /// User who last updated this entry
  pub updated_by: String,
  /// When this entry was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this entry was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpServerInfo - Nested server context in MCP instance responses
// ============================================================================

/// Minimal MCP server info embedded in MCP instance responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServerInfo {
  pub id: String,
  pub url: String,
  pub name: String,
  pub enabled: bool,
}

// ============================================================================
// McpAuthType - Authentication type for MCP instances
// ============================================================================

#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Default,
  Display,
  EnumString,
  IntoStaticStr,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum McpAuthType {
  #[default]
  Public,
  Header,
  Oauth,
}

impl McpAuthType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// McpAuthParamType - Type of auth parameter (header or query)
// ============================================================================

#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Display,
  EnumString,
  IntoStaticStr,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum McpAuthParamType {
  Header,
  Query,
}

impl McpAuthParamType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// McpAuthConfigType - Type of auth config (header or oauth)
// ============================================================================

#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Display,
  EnumString,
  IntoStaticStr,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum McpAuthConfigType {
  Header,
  Oauth,
}

impl McpAuthConfigType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// McpAuthConfigParam - Key definition response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpAuthConfigParam {
  pub id: String,
  pub param_type: McpAuthParamType,
  pub param_key: String,
}

// ============================================================================
// McpAuthConfigParamInput - Key definition input
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpAuthConfigParamInput {
  pub param_type: McpAuthParamType,
  pub param_key: String,
}

// ============================================================================
// McpAuthParam - Masked auth param response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpAuthParam {
  pub id: String,
  pub param_type: McpAuthParamType,
  pub param_key: String,
  pub has_value: bool,
}

// ============================================================================
// McpAuthParamInput - Auth param input (for creating params)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpAuthParamInput {
  pub param_type: McpAuthParamType,
  pub param_key: String,
  pub value: String,
}

// ============================================================================
// Mcp - User-owned MCP instance (public API model)
// ============================================================================

/// User-owned MCP server instance with tool caching and filtering.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Mcp {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// Server info resolved via JOIN
  pub mcp_server: McpServerInfo,
  /// User-defined slug for this instance
  pub slug: String,
  /// Human-readable name
  pub name: String,
  /// Optional description for this instance
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this instance is enabled
  pub enabled: bool,
  /// Cached tool schemas from the MCP server (JSON array)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_cache: Option<Vec<McpTool>>,
  /// Whitelisted tool names (empty = block all)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_filter: Option<Vec<String>>,
  pub auth_type: McpAuthType,
  /// Reference to the auth config (mcp_auth_configs.id)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_config_id: Option<String>,
  /// When this instance was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this instance was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpOAuthConfig - Public API model for OAuth 2.1 pre-registered client config
// ============================================================================

/// OAuth 2.1 config for pre-registered or dynamically registered client (secrets masked).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpOAuthConfig {
  pub id: String,
  /// Human-readable display name
  pub name: String,
  pub mcp_server_id: String,
  pub registration_type: RegistrationType,
  pub client_id: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_id_issued_at: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_endpoint_auth_method: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes: Option<String>,
  pub has_client_secret: bool,
  pub has_registration_access_token: bool,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpOAuthToken - Public API model for OAuth 2.1 stored token
// ============================================================================

/// OAuth 2.1 token stored for a config (secrets masked).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpOAuthToken {
  pub id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mcp_id: Option<String>,
  pub auth_config_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes_granted: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<i64>,
  pub has_refresh_token: bool,
  pub user_id: String,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpExecutionRequest / McpExecutionResponse
// ============================================================================

/// Request to execute a tool on an MCP server instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpExecutionRequest {
  /// Tool parameters as JSON
  pub params: serde_json::Value,
}

/// Response from MCP tool execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpExecutionResponse {
  /// Successful result (JSON), if any
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  /// Error message, if execution failed
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

// ============================================================================
// RegistrationType - OAuth registration type enum
// ============================================================================

/// OAuth 2.1 registration type: pre-registered client or dynamic client registration (DCR).
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Default,
  Display,
  EnumString,
  IntoStaticStr,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RegistrationType {
  #[default]
  PreRegistered,
  DynamicRegistration,
}

impl RegistrationType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// Unified auth config discriminated union types
// ============================================================================

/// Discriminated union for creating any type of MCP auth config.
/// The JSON `"type"` field determines the variant: `"header"` or `"oauth"`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CreateMcpAuthConfigRequest {
  Header {
    name: String,
    entries: Vec<McpAuthConfigParamInput>,
  },
  Oauth {
    name: String,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    scopes: Option<String>,
    /// `"pre_registered"` (default) or `"dynamic_registration"`
    #[serde(default)]
    registration_type: RegistrationType,
    #[serde(default)]
    registration_access_token: Option<String>,
    #[serde(default)]
    registration_endpoint: Option<String>,
    #[serde(default)]
    token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    client_id_issued_at: Option<i64>,
  },
}

impl validator::Validate for CreateMcpAuthConfigRequest {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      CreateMcpAuthConfigRequest::Header { .. } => Ok(()),
      CreateMcpAuthConfigRequest::Oauth {
        authorization_endpoint,
        token_endpoint,
        registration_endpoint,
        ..
      } => {
        let mut errors = validator::ValidationErrors::new();
        if !is_valid_http_url(authorization_endpoint) {
          errors.add(
            "authorization_endpoint",
            validator::ValidationError::new("invalid_url_scheme"),
          );
        }
        if !is_valid_http_url(token_endpoint) {
          errors.add(
            "token_endpoint",
            validator::ValidationError::new("invalid_url_scheme"),
          );
        }
        if let Some(ep) = registration_endpoint {
          if !is_valid_http_url(ep) {
            errors.add(
              "registration_endpoint",
              validator::ValidationError::new("invalid_url_scheme"),
            );
          }
        }
        if errors.is_empty() {
          Ok(())
        } else {
          Err(errors)
        }
      }
    }
  }
}

fn is_valid_http_url(url: &str) -> bool {
  crate::validate_http_url(url).is_ok()
}

/// Discriminated union response for any type of MCP auth config.
/// The JSON `"type"` field determines the variant: `"header"` or `"oauth"`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum McpAuthConfigResponse {
  Header {
    id: String,
    name: String,
    mcp_server_id: String,
    created_by: String,
    entries: Vec<McpAuthConfigParam>,
    #[schema(value_type = String, format = "date-time")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    updated_at: DateTime<Utc>,
  },
  Oauth {
    id: String,
    name: String,
    mcp_server_id: String,
    created_by: String,
    registration_type: RegistrationType,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    registration_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id_issued_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_endpoint_auth_method: Option<String>,
    has_client_secret: bool,
    has_registration_access_token: bool,
    #[schema(value_type = String, format = "date-time")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    updated_at: DateTime<Utc>,
  },
}

impl McpAuthConfigResponse {
  pub fn id(&self) -> &str {
    match self {
      McpAuthConfigResponse::Header { id, .. } => id,
      McpAuthConfigResponse::Oauth { id, .. } => id,
    }
  }

  pub fn mcp_server_id(&self) -> &str {
    match self {
      McpAuthConfigResponse::Header { mcp_server_id, .. } => mcp_server_id,
      McpAuthConfigResponse::Oauth { mcp_server_id, .. } => mcp_server_id,
    }
  }
}

/// List wrapper for unified auth config responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpAuthConfigsListResponse {
  pub auth_configs: Vec<McpAuthConfigResponse>,
}

// ============================================================================
// Entity -> Response conversions
// ============================================================================

impl From<super::mcp_server_entity::McpServerEntity> for McpServer {
  fn from(entity: super::mcp_server_entity::McpServerEntity) -> Self {
    Self {
      id: entity.id,
      url: entity.url,
      name: entity.name,
      description: entity.description,
      enabled: entity.enabled,
      created_by: entity.created_by,
      updated_by: entity.updated_by,
      created_at: entity.created_at,
      updated_at: entity.updated_at,
    }
  }
}

impl From<super::mcp_entity::McpWithServerEntity> for Mcp {
  fn from(row: super::mcp_entity::McpWithServerEntity) -> Self {
    let tools_cache: Option<Vec<McpTool>> = row
      .tools_cache
      .as_ref()
      .and_then(|tc| serde_json::from_str(tc).ok());
    let tools_filter: Option<Vec<String>> = row
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok());

    Self {
      id: row.id,
      mcp_server: McpServerInfo {
        id: row.mcp_server_id,
        url: row.server_url,
        name: row.server_name,
        enabled: row.server_enabled,
      },
      slug: row.slug,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      tools_cache,
      tools_filter,
      auth_type: row.auth_type,
      auth_config_id: row.auth_config_id,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}

// ============================================================================
// McpRequest - Input for creating/updating MCP instances
// ============================================================================

/// Input for creating or updating an MCP instance.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct McpRequest {
  /// Human-readable name (required)
  #[validate(
    length(min = 1, max = 100),
    custom(function = "validate_mcp_instance_name_validator")
  )]
  pub name: String,
  /// User-defined slug for this instance (1-24 chars, alphanumeric + hyphens)
  #[validate(
    length(min = 1, max = 24),
    custom(function = "validate_mcp_slug_validator")
  )]
  pub slug: String,
  /// MCP server ID (required for create, ignored for update)
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub mcp_server_id: Option<String>,
  /// Optional description
  #[serde(default)]
  #[validate(length(max = 255))]
  pub description: Option<String>,
  /// Whether this instance is enabled
  pub enabled: bool,
  /// Cached tool schemas from the MCP server (JSON array)
  #[serde(default)]
  pub tools_cache: Option<Vec<McpTool>>,
  /// Whitelisted tool names
  #[serde(default)]
  pub tools_filter: Option<Vec<String>>,
  /// Authentication type
  #[serde(default)]
  pub auth_type: McpAuthType,
  /// Reference to auth config
  #[serde(default)]
  pub auth_config_id: Option<String>,
  /// Instance-level auth params (values for the auth config's key definitions)
  #[serde(default)]
  pub credentials: Option<Vec<McpAuthParamInput>>,
  /// OAuth token ID to link to this MCP instance (set after OAuth flow)
  #[serde(default)]
  pub oauth_token_id: Option<String>,
}

// ============================================================================
// McpServerRequest - Input for creating/updating MCP servers
// ============================================================================

/// Input for creating or updating an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct McpServerRequest {
  /// MCP server endpoint URL (trimmed, case-insensitive unique)
  #[validate(
    length(min = 1, max = 2048),
    custom(function = "validate_mcp_server_url_validator")
  )]
  pub url: String,
  /// Human-readable display name
  #[validate(length(min = 1, max = 100))]
  pub name: String,
  /// Optional description
  #[serde(default)]
  #[validate(length(max = 255))]
  pub description: Option<String>,
  /// Whether this MCP server is enabled
  pub enabled: bool,
  /// Optional auth config to create alongside the server (create only)
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub auth_config: Option<CreateMcpAuthConfigRequest>,
}

// ============================================================================
// Validator-derive wrapper functions
// ============================================================================

fn validate_mcp_instance_name_validator(_name: &str) -> Result<(), validator::ValidationError> {
  // Length checks are handled by #[validate(length(...))]; this is for custom logic only.
  Ok(())
}

fn validate_mcp_slug_validator(slug: &str) -> Result<(), validator::ValidationError> {
  if !MCP_SLUG_REGEX.is_match(slug) {
    return Err(validator::ValidationError::new("invalid_mcp_slug"));
  }
  Ok(())
}

fn validate_mcp_server_url_validator(url: &str) -> Result<(), validator::ValidationError> {
  crate::validate_http_url(url)
}

// ============================================================================
// Validation functions
// ============================================================================

static MCP_SLUG_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9-]+$").expect("Invalid MCP slug regex"));

pub const MAX_MCP_SLUG_LEN: usize = 24;
pub const MAX_MCP_INSTANCE_NAME_LEN: usize = 100;
pub const MAX_MCP_DESCRIPTION_LEN: usize = 255;
pub const MAX_MCP_SERVER_NAME_LEN: usize = 100;
pub const MAX_MCP_SERVER_URL_LEN: usize = 2048;
pub const MAX_MCP_AUTH_CONFIG_NAME_LEN: usize = 100;

#[cfg(test)]
#[path = "test_mcp_objs_validation.rs"]
mod test_mcp_objs_validation;

#[cfg(test)]
#[path = "test_mcp_objs_types.rs"]
mod test_mcp_objs_types;
