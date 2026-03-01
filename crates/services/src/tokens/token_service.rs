use async_trait::async_trait;
use std::sync::Arc;

use super::ApiToken;
use crate::db::{DbError, DbService};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait TokenService: Send + Sync + std::fmt::Debug {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError>;

  async fn get_api_token_by_id(&self, user_id: &str, id: &str)
    -> Result<Option<ApiToken>, DbError>;

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError>;

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError>;
}

#[derive(Debug, derive_new::new)]
pub struct DefaultTokenService {
  db_service: Arc<dyn DbService>,
}

#[async_trait]
impl TokenService for DefaultTokenService {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    self.db_service.create_api_token(token).await
  }

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError> {
    self
      .db_service
      .list_api_tokens(user_id, page, per_page)
      .await
  }

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, DbError> {
    self.db_service.get_api_token_by_id(user_id, id).await
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
    self.db_service.get_api_token_by_prefix(prefix).await
  }

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError> {
    self.db_service.update_api_token(user_id, token).await
  }
}

#[cfg(test)]
#[path = "test_token_service.rs"]
mod test_token_service;
