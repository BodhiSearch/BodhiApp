use crate::db::{DbError, UserAccessRequest, UserAccessRequestStatus};

#[async_trait::async_trait]
pub trait AccessRepository: Send + Sync {
  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError>;

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError>;

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError>;

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError>;

  async fn update_request_status(
    &self,
    id: i64,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError>;

  async fn get_request_by_id(&self, id: i64) -> Result<Option<UserAccessRequest>, DbError>;
}
