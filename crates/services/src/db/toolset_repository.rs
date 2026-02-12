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

  async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError>;

  async fn update_toolset(
    &self,
    row: &ToolsetRow,
    api_key_update: ApiKeyUpdate,
  ) -> Result<ToolsetRow, DbError>;

  async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError>;

  async fn list_toolsets_by_toolset_type(
    &self,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetRow>, DbError>;

  async fn delete_toolset(&self, id: &str) -> Result<(), DbError>;

  async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError>;

  // App-level toolset type config
  async fn set_app_toolset_enabled(
    &self,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigRow, DbError>;

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError>;

  async fn get_app_toolset_config(
    &self,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError>;

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
