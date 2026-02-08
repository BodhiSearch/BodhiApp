use crate::db::{ApiKeyUpdate, DownloadRequest, ModelMetadataRow};
use crate::db::DbError;
use chrono::{DateTime, Utc};
use objs::{ApiAlias};

#[async_trait::async_trait]
pub trait ModelRepository: Send + Sync {
  // Downloads
  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError>;

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError>;

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError>;

  // API Model Aliases
  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError>;

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError>;

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError>;

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError>;

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError>;

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;

  // Model Metadata
  async fn upsert_model_metadata(
    &self,
    metadata: &ModelMetadataRow,
  ) -> Result<(), DbError>;

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataRow>, DbError>;

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<
    std::collections::HashMap<(String, String, String), ModelMetadataRow>,
    DbError,
  >;

  async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError>;
}
