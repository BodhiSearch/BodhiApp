use crate::db::{DbError, DefaultDbService};
use crate::mcps::{McpAuthType, McpRow, McpWithServerRow};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};

use super::mcp_entity;
use super::mcp_server_entity;

#[async_trait::async_trait]
pub trait McpInstanceRepository: Send + Sync {
  // MCP user instances
  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError>;

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError>;

  async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError>;

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError>;
}

#[async_trait::async_trait]
impl McpInstanceRepository for DefaultDbService {
  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    let active = mcp_entity::ActiveModel {
      id: Set(row.id.clone()),
      created_by: Set(row.created_by.clone()),
      mcp_server_id: Set(row.mcp_server_id.clone()),
      name: Set(row.name.clone()),
      slug: Set(row.slug.clone()),
      description: Set(row.description.clone()),
      enabled: Set(row.enabled),
      tools_cache: Set(row.tools_cache.clone()),
      tools_filter: Set(row.tools_filter.clone()),
      auth_type: Set(row.auth_type.clone()),
      auth_uuid: Set(row.auth_uuid.clone()),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(McpRow::from(model))
  }

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError> {
    let result = mcp_entity::Entity::find_by_id(id)
      .filter(mcp_entity::Column::CreatedBy.eq(user_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpRow::from))
  }

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError> {
    let result = mcp_entity::Entity::find()
      .filter(mcp_entity::Column::CreatedBy.eq(user_id))
      .filter(mcp_entity::Column::Slug.eq(slug))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpRow::from))
  }

  async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError> {
    use sea_orm::FromQueryResult;

    #[derive(FromQueryResult)]
    struct McpWithServer {
      id: String,
      created_by: String,
      mcp_server_id: String,
      name: String,
      slug: String,
      description: Option<String>,
      enabled: bool,
      tools_cache: Option<String>,
      tools_filter: Option<String>,
      auth_type: McpAuthType,
      auth_uuid: Option<String>,
      created_at: chrono::DateTime<chrono::Utc>,
      updated_at: chrono::DateTime<chrono::Utc>,
      server_url: String,
      server_name: String,
      server_enabled: bool,
    }

    let results = mcp_entity::Entity::find()
      .select_only()
      .columns([
        mcp_entity::Column::Id,
        mcp_entity::Column::CreatedBy,
        mcp_entity::Column::McpServerId,
        mcp_entity::Column::Name,
        mcp_entity::Column::Slug,
        mcp_entity::Column::Description,
        mcp_entity::Column::Enabled,
        mcp_entity::Column::ToolsCache,
        mcp_entity::Column::ToolsFilter,
        mcp_entity::Column::AuthType,
        mcp_entity::Column::AuthUuid,
        mcp_entity::Column::CreatedAt,
        mcp_entity::Column::UpdatedAt,
      ])
      .column_as(mcp_server_entity::Column::Url, "server_url")
      .column_as(mcp_server_entity::Column::Name, "server_name")
      .column_as(mcp_server_entity::Column::Enabled, "server_enabled")
      .join(
        sea_orm::JoinType::InnerJoin,
        mcp_entity::Entity::belongs_to(mcp_server_entity::Entity)
          .from(mcp_entity::Column::McpServerId)
          .to(mcp_server_entity::Column::Id)
          .into(),
      )
      .filter(mcp_entity::Column::CreatedBy.eq(user_id))
      .into_model::<McpWithServer>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(
      results
        .into_iter()
        .map(|r| McpWithServerRow {
          id: r.id,
          created_by: r.created_by,
          mcp_server_id: r.mcp_server_id,
          name: r.name,
          slug: r.slug,
          description: r.description,
          enabled: r.enabled,
          tools_cache: r.tools_cache,
          tools_filter: r.tools_filter,
          auth_type: r.auth_type,
          auth_uuid: r.auth_uuid,
          created_at: r.created_at,
          updated_at: r.updated_at,
          server_url: r.server_url,
          server_name: r.server_name,
          server_enabled: r.server_enabled,
        })
        .collect(),
    )
  }

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    let active = mcp_entity::ActiveModel {
      id: Set(row.id.clone()),
      name: Set(row.name.clone()),
      slug: Set(row.slug.clone()),
      description: Set(row.description.clone()),
      enabled: Set(row.enabled),
      tools_cache: Set(row.tools_cache.clone()),
      tools_filter: Set(row.tools_filter.clone()),
      auth_type: Set(row.auth_type.clone()),
      auth_uuid: Set(row.auth_uuid.clone()),
      updated_at: Set(row.updated_at),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(McpRow::from(model))
  }

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    mcp_entity::Entity::delete_many()
      .filter(mcp_entity::Column::CreatedBy.eq(user_id))
      .filter(mcp_entity::Column::Id.eq(id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }
}

#[cfg(test)]
#[path = "test_mcp_instance_repository.rs"]
mod test_mcp_instance_repository;
