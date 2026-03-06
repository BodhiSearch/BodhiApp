use crate::{
  AppService, AuthContext, DownloadRequestEntity, DownloadServiceError, DownloadStatus,
  NewDownloadRequest, PaginatedDownloadResponse,
};
use std::sync::Arc;

/// Auth-scoped wrapper around DownloadService that injects tenant_id from AuthContext.
pub struct AuthScopedDownloadService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedDownloadService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  /// Create a new download request
  pub async fn create(
    &self,
    form: &NewDownloadRequest,
  ) -> Result<DownloadRequestEntity, DownloadServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .download_service()
      .create(tenant_id, form)
      .await
  }

  /// Get a specific download request by ID
  pub async fn get(&self, id: &str) -> Result<DownloadRequestEntity, DownloadServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self.app_service.download_service().get(tenant_id, id).await
  }

  /// List download requests with pagination
  pub async fn list(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedDownloadResponse, DownloadServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .download_service()
      .list(tenant_id, page, page_size)
      .await
  }

  /// Find existing download request by repo and filename
  pub async fn find_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Option<DownloadRequestEntity>, DownloadServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .download_service()
      .find_by_repo_filename(tenant_id, repo, filename)
      .await
  }

  /// Update the status of a download request
  pub async fn update_status(
    &self,
    id: &str,
    status: DownloadStatus,
    error: Option<String>,
  ) -> Result<(), DownloadServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .download_service()
      .update_status(tenant_id, id, status, error)
      .await
  }
}
