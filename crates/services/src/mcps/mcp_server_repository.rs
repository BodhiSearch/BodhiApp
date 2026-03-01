use crate::db::{DbError, DefaultDbService};
use crate::mcps::McpServerRow;
use sea_orm::{
  sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set,
};

use super::mcp_entity;
use super::mcp_server_entity;

#[async_trait::async_trait]
pub trait McpServerRepository: Send + Sync {
  // MCP server registry (admin-managed)
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;

  async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError>;

  /// Returns (enabled_count, disabled_count) for MCPs referencing this server
  async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError>;

  /// Clear tools_cache and tools_filter on all MCPs linked to a server
  async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError>;
}

#[async_trait::async_trait]
impl McpServerRepository for DefaultDbService {
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    let active = mcp_server_entity::ActiveModel {
      id: Set(row.id.clone()),
      url: Set(row.url.clone()),
      name: Set(row.name.clone()),
      description: Set(row.description.clone()),
      enabled: Set(row.enabled),
      created_by: Set(row.created_by.clone()),
      updated_by: Set(row.updated_by.clone()),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(McpServerRow::from(model))
  }

  async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    let active = mcp_server_entity::ActiveModel {
      id: Set(row.id.clone()),
      url: Set(row.url.clone()),
      name: Set(row.name.clone()),
      description: Set(row.description.clone()),
      enabled: Set(row.enabled),
      updated_by: Set(row.updated_by.clone()),
      updated_at: Set(row.updated_at),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(McpServerRow::from(model))
  }

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError> {
    let result = mcp_server_entity::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpServerRow::from))
  }

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError> {
    let result = mcp_server_entity::Entity::find()
      .filter(mcp_server_entity::Column::Url.eq(url))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpServerRow::from))
  }

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError> {
    let mut query = mcp_server_entity::Entity::find();
    if let Some(e) = enabled {
      query = query.filter(mcp_server_entity::Column::Enabled.eq(e));
    }
    let results = query.all(&self.db).await.map_err(DbError::from)?;
    Ok(results.into_iter().map(McpServerRow::from).collect())
  }

  async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError> {
    use sea_orm::FromQueryResult;

    #[derive(FromQueryResult)]
    struct CountResult {
      enabled_count: i64,
      disabled_count: i64,
    }

    let result = mcp_entity::Entity::find()
      .select_only()
      .column_as(
        Expr::cust("COALESCE(SUM(CASE WHEN enabled THEN 1 ELSE 0 END), 0)"),
        "enabled_count",
      )
      .column_as(
        Expr::cust("COALESCE(SUM(CASE WHEN NOT enabled THEN 1 ELSE 0 END), 0)"),
        "disabled_count",
      )
      .filter(mcp_entity::Column::McpServerId.eq(server_id))
      .into_model::<CountResult>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(r) => Ok((r.enabled_count, r.disabled_count)),
      None => Ok((0, 0)),
    }
  }

  async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError> {
    let now = self.time_service.utc_now();
    let result = mcp_entity::Entity::update_many()
      .col_expr(
        mcp_entity::Column::ToolsCache,
        Expr::value(sea_orm::Value::String(None)),
      )
      .col_expr(
        mcp_entity::Column::ToolsFilter,
        Expr::value(sea_orm::Value::String(None)),
      )
      .col_expr(mcp_entity::Column::UpdatedAt, Expr::value(now))
      .filter(mcp_entity::Column::McpServerId.eq(server_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.rows_affected)
  }
}

#[cfg(test)]
#[path = "test_mcp_server_repository.rs"]
mod test_mcp_server_repository;
