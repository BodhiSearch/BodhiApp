use crate::db::{DbError, DefaultDbService};
use crate::models::download_request_entity as download_request;
use crate::models::DownloadRequest;
use sea_orm::prelude::*;
use sea_orm::{NotSet, QueryOrder, QuerySelect, Set};

#[async_trait::async_trait]
pub trait DownloadRepository: Send + Sync {
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
}

#[async_trait::async_trait]
impl DownloadRepository for DefaultDbService {
  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let model = download_request::ActiveModel {
      id: Set(request.id.clone()),
      repo: Set(request.repo.clone()),
      filename: Set(request.filename.clone()),
      status: Set(request.status.clone()),
      error: Set(request.error.clone()),
      total_bytes: Set(request.total_bytes),
      downloaded_bytes: Set(request.downloaded_bytes),
      started_at: Set(request.started_at),
      created_at: Set(request.created_at),
      updated_at: Set(request.updated_at),
    };
    download_request::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = download_request::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result)
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let model = download_request::ActiveModel {
      id: Set(request.id.clone()),
      repo: NotSet,
      filename: NotSet,
      status: Set(request.status.clone()),
      error: Set(request.error.clone()),
      total_bytes: Set(request.total_bytes),
      downloaded_bytes: Set(request.downloaded_bytes),
      started_at: Set(request.started_at),
      created_at: NotSet,
      updated_at: Set(request.updated_at),
    };
    download_request::Entity::update(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let total = download_request::Entity::find()
      .count(&self.db)
      .await
      .map_err(DbError::from)? as usize;

    let results = download_request::Entity::find()
      .order_by_desc(download_request::Column::UpdatedAt)
      .offset(((page - 1) * page_size) as u64)
      .limit(page_size as u64)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok((results, total))
  }

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError> {
    let results = download_request::Entity::find()
      .filter(download_request::Column::Repo.eq(repo))
      .filter(download_request::Column::Filename.eq(filename))
      .order_by_desc(download_request::Column::CreatedAt)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results)
  }
}
