use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ulid::Ulid;
use utoipa::ToSchema;
use validator::Validate;

use crate::db::{DbError, DbService, TimeService};
use crate::models::{DownloadRequestEntity, DownloadStatus};
use errmeta::{AppError, EntityError, ErrorType};

// =============================================================================
// NewDownloadRequest (input type)
// =============================================================================

/// Request for creating a new download request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
    "filename": "mistral-7b-instruct-v0.1.Q4_K_M.gguf"
}))]
pub struct NewDownloadRequest {
  /// HuggingFace repository name in format 'username/repository-name'
  #[schema(
    pattern = "^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$",
    example = "TheBloke/Mistral-7B-Instruct-v0.1-GGUF"
  )]
  pub repo: String,
  /// Model file name to download (typically .gguf format)
  #[schema(
    pattern = ".*\\.(gguf|bin|safetensors)$",
    example = "mistral-7b-instruct-v0.1.Q4_K_M.gguf"
  )]
  pub filename: String,
}

// =============================================================================
// Download (output type — entity minus tenant_id)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DownloadRequest {
  pub id: String,
  pub repo: String,
  pub filename: String,
  pub status: DownloadStatus,
  pub error: Option<String>,
  pub total_bytes: Option<i64>,
  pub downloaded_bytes: i64,
  #[schema(value_type = Option<String>, format = "date-time")]
  pub started_at: Option<DateTime<Utc>>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

// =============================================================================
// PaginatedDownloadResponse (output type for list)
// =============================================================================

/// Paginated list of download requests
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedDownloadResponse {
  pub data: Vec<DownloadRequest>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

// =============================================================================
// DownloadServiceError
// =============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DownloadServiceError {
  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::NotFound, code = "download_service_error-not_found")]
  NotFound(#[from] EntityError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "download_service_error-auth")]
  Auth(#[from] crate::auth::AuthContextError),
}

// =============================================================================
// DownloadService trait
// =============================================================================

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait DownloadService: Send + Sync + std::fmt::Debug {
  /// Create a new download request
  async fn create(
    &self,
    tenant_id: &str,
    form: &NewDownloadRequest,
  ) -> Result<DownloadRequestEntity, DownloadServiceError>;

  /// Get a specific download request by ID
  async fn get(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<DownloadRequestEntity, DownloadServiceError>;

  /// List download requests with pagination
  async fn list(
    &self,
    tenant_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedDownloadResponse, DownloadServiceError>;

  /// Find existing download requests by repo and filename
  async fn find_by_repo_filename(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
  ) -> Result<Option<DownloadRequestEntity>, DownloadServiceError>;

  /// Update the status of a download request (used by background download task)
  async fn update_status(
    &self,
    tenant_id: &str,
    id: &str,
    status: DownloadStatus,
    error: Option<String>,
  ) -> Result<(), DownloadServiceError>;
}

// =============================================================================
// DefaultDownloadService
// =============================================================================

#[derive(Debug, derive_new::new)]
pub struct DefaultDownloadService {
  db_service: Arc<dyn DbService>,
  time_service: Arc<dyn TimeService>,
}

#[async_trait]
impl DownloadService for DefaultDownloadService {
  async fn create(
    &self,
    tenant_id: &str,
    form: &NewDownloadRequest,
  ) -> Result<DownloadRequestEntity, DownloadServiceError> {
    let now = self.time_service.utc_now();
    let id = Ulid::new().to_string();

    let model = DownloadRequestEntity {
      id,
      tenant_id: tenant_id.to_string(),
      repo: form.repo.clone(),
      filename: form.filename.clone(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: now,
      updated_at: now,
      total_bytes: None,
      downloaded_bytes: 0,
      started_at: None,
    };

    self.db_service.create_download_request(&model).await?;
    Ok(model)
  }

  async fn get(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<DownloadRequestEntity, DownloadServiceError> {
    self
      .db_service
      .get_download_request(tenant_id, id)
      .await?
      .ok_or_else(|| {
        EntityError::NotFound(format!(
          "item '{}' of type 'download_requests' not found in db",
          id
        ))
        .into()
      })
  }

  async fn list(
    &self,
    tenant_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedDownloadResponse, DownloadServiceError> {
    let (data, total) = self
      .db_service
      .list_download_requests(tenant_id, page, page_size)
      .await?;

    Ok(PaginatedDownloadResponse {
      data: data.into_iter().map(Into::into).collect(),
      total,
      page,
      page_size,
    })
  }

  async fn find_by_repo_filename(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
  ) -> Result<Option<DownloadRequestEntity>, DownloadServiceError> {
    let results = self
      .db_service
      .find_download_request_by_repo_filename(tenant_id, repo, filename)
      .await?;

    Ok(
      results
        .into_iter()
        .find(|r| r.repo == repo && r.filename == filename),
    )
  }

  async fn update_status(
    &self,
    tenant_id: &str,
    id: &str,
    status: DownloadStatus,
    error: Option<String>,
  ) -> Result<(), DownloadServiceError> {
    let mut request = self
      .db_service
      .get_download_request(tenant_id, id)
      .await?
      .ok_or_else(|| {
        EntityError::NotFound(format!(
          "item '{}' of type 'download_requests' not found in db",
          id
        ))
      })?;

    request.status = status;
    request.error = error;
    request.updated_at = self.time_service.utc_now();

    self.db_service.update_download_request(&request).await?;
    Ok(())
  }
}
