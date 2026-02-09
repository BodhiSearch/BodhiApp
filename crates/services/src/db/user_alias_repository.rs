use crate::db::DbError;
use objs::UserAlias;

#[async_trait::async_trait]
pub trait UserAliasRepository: Send + Sync {
  async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError>;
  async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError>;
  async fn get_user_alias_by_name(&self, alias: &str) -> Result<Option<UserAlias>, DbError>;
  async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError>;
  async fn delete_user_alias(&self, id: &str) -> Result<(), DbError>;
  async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError>;
}
