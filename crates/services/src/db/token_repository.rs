use crate::db::{ApiToken, DbError};

#[async_trait::async_trait]
pub trait TokenRepository: Send + Sync {
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
