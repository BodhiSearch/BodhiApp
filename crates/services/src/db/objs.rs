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
  /// Email of the requesting user
  pub email: String,
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
  pub token_id: String,
  pub token_hash: String,
  pub status: TokenStatus,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod test {
  use crate::db::{DownloadRequest, DownloadStatus};
  use chrono::Utc;

  #[test]
  fn test_download_request_new_pending_initializes_progress_fields() {
    let request = DownloadRequest::new_pending("test/repo", "test.gguf", Utc::now());
    assert_eq!(
      DownloadRequest {
        id: request.id.clone(),
        repo: "test/repo".to_string(),
        filename: "test.gguf".to_string(),
        status: DownloadStatus::Pending,
        error: None,
        created_at: request.created_at.clone(),
        updated_at: request.updated_at.clone(),
        total_bytes: None,
        downloaded_bytes: 0,
        started_at: None,
      },
      request
    );
  }
}
