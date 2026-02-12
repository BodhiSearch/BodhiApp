use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use objs::{is_default, BuilderError};
use serde::{Deserialize, Serialize};

use strum::EnumString;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DownloadStatus {
  Pending,
  Completed,
  Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DownloadRequest {
  pub id: String,
  pub repo: String,
  pub filename: String,
  pub status: DownloadStatus,
  pub error: Option<String>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
  pub total_bytes: Option<u64>,
  #[serde(default)]
  pub downloaded_bytes: u64,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub started_at: Option<DateTime<Utc>>,
}

impl DownloadRequest {
  pub fn new_pending(repo: &str, filename: &str, now: DateTime<Utc>) -> Self {
    DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: repo.to_string(),
      filename: filename.to_string(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: now,
      updated_at: now,
      total_bytes: None,
      downloaded_bytes: 0,
      started_at: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserAccessRequest {
  /// Unique identifier for the request
  pub id: i64,
  /// Username of the requesting user
  pub username: String,
  /// User ID (UUID) of the requesting user
  pub user_id: String,
  #[serde(default)]
  pub reviewer: Option<String>,
  /// Current status of the request
  pub status: UserAccessRequestStatus,
  /// Creation timestamp
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum UserAccessRequestStatus {
  Pending,
  Approved,
  Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum TokenStatus {
  Active,
  Inactive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ApiToken {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub token_prefix: String,
  pub token_hash: String,
  pub scopes: String,
  pub status: TokenStatus,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

/// Represents an API key update operation for API model aliases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyUpdate {
  /// Keep the existing API key unchanged
  Keep,
  /// Set a new API key (or add one if none exists) - Option<String> supports both setting and clearing
  Set(Option<String>),
}

/// Model metadata row stored in database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sqlx::FromRow)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  derive(Default, derive_builder::Builder)
)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(setter(into, strip_option), default, build_fn(error = BuilderError))
)]
pub struct ModelMetadataRow {
  pub id: i64,
  pub source: String,
  pub repo: Option<String>,
  pub filename: Option<String>,
  pub snapshot: Option<String>,
  pub api_model_id: Option<String>,
  pub capabilities_vision: Option<i64>,
  pub capabilities_audio: Option<i64>,
  pub capabilities_thinking: Option<i64>,
  pub capabilities_function_calling: Option<i64>,
  pub capabilities_structured_output: Option<i64>,
  pub context_max_input_tokens: Option<i64>,
  pub context_max_output_tokens: Option<i64>,
  pub architecture: Option<String>,
  pub additional_metadata: Option<String>,
  pub chat_template: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub extracted_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<ModelMetadataRow> for objs::ModelMetadata {
  fn from(row: ModelMetadataRow) -> Self {
    // Parse architecture from the architecture JSON column (not additional_metadata)
    let architecture: Option<objs::ModelArchitecture> = row
      .architecture
      .as_ref()
      .and_then(|s| serde_json::from_str(s).ok());

    objs::ModelMetadata {
      capabilities: objs::ModelCapabilities {
        vision: row.capabilities_vision.map(|v| v != 0),
        audio: row.capabilities_audio.map(|v| v != 0),
        thinking: row.capabilities_thinking.map(|v| v != 0),
        tools: objs::ToolCapabilities {
          function_calling: row.capabilities_function_calling.map(|v| v != 0),
          structured_output: row.capabilities_structured_output.map(|v| v != 0),
        },
      },
      context: objs::ContextLimits {
        max_input_tokens: row.context_max_input_tokens.map(|v| v as u64),
        max_output_tokens: row.context_max_output_tokens.map(|v| v as u64),
      },
      architecture: architecture.unwrap_or_else(|| objs::ModelArchitecture {
        family: None,
        parameter_count: None,
        quantization: None,
        format: "gguf".to_string(),
      }),
      chat_template: row.chat_template,
    }
  }
}

// ============================================================================
// ToolsetRow - Database row for user toolset instances
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ToolsetRow {
  pub id: String,
  pub user_id: String,
  pub toolset_type: String,
  pub name: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub encrypted_api_key: Option<String>,
  pub salt: Option<String>,
  pub nonce: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

// ============================================================================
// AppClientToolsetConfigRow - Database row for cached app-client toolset configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppClientToolsetConfigRow {
  pub id: i64,
  pub app_client_id: String,
  pub config_version: Option<String>,
  pub toolsets_json: String,
  pub resource_scope: String,
  pub created_at: i64,
  pub updated_at: i64,
}

// ============================================================================
// AppToolsetConfigRow - Database row for app-level toolset type configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AppToolsetConfigRow {
  pub toolset_type: String,
  pub enabled: bool,
  pub updated_by: String,
  pub created_at: i64,
  pub updated_at: i64,
}

// ============================================================================
// AppAccessRequestRow - Database row for app access request consent tracking
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AppAccessRequestRow {
  pub id: String,              // UUID (access_request_id)
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub flow_type: String,       // "redirect" | "popup"
  pub redirect_uri: Option<String>,
  pub status: String,          // "draft" | "approved" | "denied" | "failed"
  pub requested: String,       // JSON: {"toolset_types": [{"tool_type":"..."}]}
  pub approved: Option<String>, // JSON: {"toolset_types": [{"tool_type":"...", "status":"approved", "instance_id":"..."}]}
  pub user_id: Option<String>,
  pub resource_scope: Option<String>,         // KC-returned scope
  pub access_request_scope: Option<String>,   // KC-returned scope (NULL for auto-approve)
  pub error_message: Option<String>,          // Error details for 'failed' status
  pub expires_at: i64,         // Unix timestamp
  pub created_at: i64,
  pub updated_at: i64,
}
