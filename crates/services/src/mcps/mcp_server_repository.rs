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
  async fn create_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError>;

  async fn update_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError>;

  async fn get_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpServerRow>, DbError>;

  async fn get_mcp_server_by_url(
    &self,
    tenant_id: &str,
    url: &str,
  ) -> Result<Option<McpServerRow>, DbError>;

  async fn list_mcp_servers(
    &self,
    tenant_id: &str,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerRow>, DbError>;

  /// Returns (enabled_count, disabled_count) for MCPs referencing this server
  async fn count_mcps_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<(i64, i64), DbError>;

  /// Clear tools_cache and tools_filter on all MCPs linked to a server
  async fn clear_mcp_tools_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<u64, DbError>;
}

#[async_trait::async_trait]
impl McpServerRepository for DefaultDbService {
  async fn create_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_server_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(tenant_id_owned),
            url: Set(row.url.clone()),
            name: Set(row.name.clone()),
            description: Set(row.description.clone()),
            enabled: Set(row.enabled),
            created_by: Set(row.created_by.clone()),
            updated_by: Set(row.updated_by.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpServerRow::from(model))
        })
      })
      .await
  }

  async fn update_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Verify tenant ownership before update
          let existing = mcp_server_entity::Entity::find_by_id(row.id.clone())
            .filter(mcp_server_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          if existing.is_none() {
            return Err(DbError::ItemNotFound {
              id: row.id.clone(),
              item_type: "mcp_server".to_string(),
            });
          }
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
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(McpServerRow::from(model))
        })
      })
      .await
  }

  async fn get_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpServerRow>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_server_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_server_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn get_mcp_server_by_url(
    &self,
    tenant_id: &str,
    url: &str,
  ) -> Result<Option<McpServerRow>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let url_owned = url.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_server_entity::Entity::find()
            .filter(mcp_server_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_server_entity::Column::Url.eq(&url_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn list_mcp_servers(
    &self,
    tenant_id: &str,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerRow>, DbError> {
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = mcp_server_entity::Entity::find()
            .filter(mcp_server_entity::Column::TenantId.eq(&tenant_id_owned));
          if let Some(e) = enabled {
            query = query.filter(mcp_server_entity::Column::Enabled.eq(e));
          }
          let results = query.all(txn).await.map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn count_mcps_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<(i64, i64), DbError> {
    use sea_orm::FromQueryResult;

    #[derive(FromQueryResult)]
    struct CountResult {
      enabled_count: i64,
      disabled_count: i64,
    }

    let tenant_id_owned = tenant_id.to_string();
    let server_id_owned = server_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = mcp_entity::Entity::find()
            .select_only()
            .column_as(
              Expr::cust("COALESCE(SUM(CASE WHEN enabled THEN 1 ELSE 0 END), 0)"),
              "enabled_count",
            )
            .column_as(
              Expr::cust("COALESCE(SUM(CASE WHEN NOT enabled THEN 1 ELSE 0 END), 0)"),
              "disabled_count",
            )
            .filter(mcp_entity::Column::McpServerId.eq(&server_id_owned));

          if !tenant_id_owned.is_empty() {
            query = query.filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned));
          }

          let result = query
            .into_model::<CountResult>()
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(r) => Ok((r.enabled_count, r.disabled_count)),
            None => Ok((0, 0)),
          }
        })
      })
      .await
  }

  async fn clear_mcp_tools_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<u64, DbError> {
    let now = self.time_service.utc_now();
    let server_id_owned = server_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
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
            .filter(mcp_entity::Column::McpServerId.eq(&server_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.rows_affected)
        })
      })
      .await
  }
}

#[cfg(test)]
#[path = "test_mcp_server_repository.rs"]
mod test_mcp_server_repository;
