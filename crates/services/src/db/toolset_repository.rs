use crate::db::{
  ApiKeyUpdate, AppClientToolsetConfigRow, AppToolsetConfigRow, DbError, ToolsetRow,
};

#[async_trait::async_trait]
pub trait ToolsetRepository: Send + Sync {
  // Toolset instances
  async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError>;

  async fn get_toolset_by_name(
    &self,
    user_id: &str,
    name: &str,
  ) -> Result<Option<ToolsetRow>, DbError>;

  async fn create_toolset(
    &self,
    row: &ToolsetRow,
  ) -> Result<ToolsetRow, DbError>;

  async fn update_toolset(
    &self,
    row: &ToolsetRow,
    api_key_update: ApiKeyUpdate,
  ) -> Result<ToolsetRow, DbError>;

  async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError>;

  async fn list_toolsets_by_scope_uuid(
    &self,
    user_id: &str,
    scope_uuid: &str,
  ) -> Result<Vec<ToolsetRow>, DbError>;

  async fn delete_toolset(&self, id: &str) -> Result<(), DbError>;

  async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError>;

  // App-level toolset configuration
  async fn get_app_toolset_config_by_scope_uuid(
    &self,
    scope_uuid: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError>;

  async fn get_app_toolset_config_by_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError>;

  async fn upsert_app_toolset_config(
    &self,
    config: &AppToolsetConfigRow,
  ) -> Result<AppToolsetConfigRow, DbError>;

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError>;

  async fn list_app_toolset_configs_by_scopes(
    &self,
    scopes: &[String],
  ) -> Result<Vec<AppToolsetConfigRow>, DbError>;

  // App-Client toolset config
  async fn get_app_client_toolset_config(
    &self,
    app_client_id: &str,
  ) -> Result<Option<AppClientToolsetConfigRow>, DbError>;

  async fn upsert_app_client_toolset_config(
    &self,
    config: &AppClientToolsetConfigRow,
  ) -> Result<AppClientToolsetConfigRow, DbError>;
}
