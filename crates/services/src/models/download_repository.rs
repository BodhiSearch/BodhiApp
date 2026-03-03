use crate::db::{DbError, DefaultDbService};
use crate::models::download_request_entity as download_request;
use crate::models::DownloadRequestEntity;
use sea_orm::prelude::*;
use sea_orm::{NotSet, QueryOrder, QuerySelect, Set};

#[async_trait::async_trait]
pub trait DownloadRepository: Send + Sync {
  async fn create_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError>;

  async fn get_download_request(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<DownloadRequestEntity>, DbError>;

  async fn update_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError>;

  async fn list_download_requests(
    &self,
    tenant_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequestEntity>, usize), DbError>;

  async fn find_download_request_by_repo_filename(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequestEntity>, DbError>;
}

#[async_trait::async_trait]
impl DownloadRepository for DefaultDbService {
  async fn create_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError> {
    let tenant_id = request.tenant_id.clone();
    let request = request.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let model = download_request::ActiveModel {
            id: Set(request.id.clone()),
            tenant_id: Set(request.tenant_id.clone()),
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
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_download_request(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<DownloadRequestEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = download_request::Entity::find_by_id(id_owned)
            .filter(download_request::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn update_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError> {
    let tenant_id = request.tenant_id.clone();
    let request = request.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let model = download_request::ActiveModel {
            id: Set(request.id.clone()),
            tenant_id: NotSet,
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
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn list_download_requests(
    &self,
    tenant_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequestEntity>, usize), DbError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let total = download_request::Entity::find()
            .filter(download_request::Column::TenantId.eq(&tenant_id_owned))
            .count(txn)
            .await
            .map_err(DbError::from)? as usize;

          let results = download_request::Entity::find()
            .filter(download_request::Column::TenantId.eq(&tenant_id_owned))
            .order_by_desc(download_request::Column::UpdatedAt)
            .offset(((page - 1) * page_size) as u64)
            .limit(page_size as u64)
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok((results, total))
        })
      })
      .await
  }

  async fn find_download_request_by_repo_filename(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequestEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let repo_owned = repo.to_string();
    let filename_owned = filename.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = download_request::Entity::find()
            .filter(download_request::Column::TenantId.eq(&tenant_id_owned))
            .filter(download_request::Column::Repo.eq(&repo_owned))
            .filter(download_request::Column::Filename.eq(&filename_owned))
            .order_by_desc(download_request::Column::CreatedAt)
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok(results)
        })
      })
      .await
  }
}
