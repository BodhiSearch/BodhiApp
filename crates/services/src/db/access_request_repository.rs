use async_trait::async_trait;

use crate::db::error::DbError;
use crate::db::objs::AppAccessRequestRow;

#[async_trait]
pub trait AccessRequestRepository: Send + Sync {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError>;

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError>;

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,             // JSON string
    resource_scope: &str,
    access_request_scope: Option<String>, // NULL for auto-approve
  ) -> Result<AppAccessRequestRow, DbError>;

  async fn update_denial(
    &self,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequestRow, DbError>;

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError>;
}
