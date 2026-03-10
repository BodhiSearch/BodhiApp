use crate::db::{DbError, DefaultDbService};
use crate::mcps::{McpAuthType, McpEntity, McpWithServerEntity};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};

use super::mcp_entity;
use super::mcp_server_entity;

#[async_trait::async_trait]
pub trait McpInstanceRepository: Send + Sync {
  // MCP user instances
  async fn create_mcp(&self, tenant_id: &str, row: &McpEntity) -> Result<McpEntity, DbError>;

  async fn get_mcp(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpEntity>, DbError>;

  async fn get_mcp_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<McpEntity>, DbError>;

  async fn list_mcps_with_server(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpWithServerEntity>, DbError>;

  async fn update_mcp(&self, tenant_id: &str, row: &McpEntity) -> Result<McpEntity, DbError>;

  async fn delete_mcp(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError>;
}

#[async_trait::async_trait]
impl McpInstanceRepository for DefaultDbService {
  async fn create_mcp(&self, tenant_id: &str, row: &McpEntity) -> Result<McpEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(row.user_id.clone()),
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
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpEntity::from(model))
        })
      })
      .await
  }

  async fn get_mcp(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_entity::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn get_mcp_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<McpEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let slug_owned = slug.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_entity::Entity::find()
            .filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_entity::Column::UserId.eq(&user_id_owned))
            .filter(mcp_entity::Column::Slug.eq(&slug_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn list_mcps_with_server(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpWithServerEntity>, DbError> {
    use sea_orm::FromQueryResult;

    #[derive(FromQueryResult)]
    struct McpWithServer {
      id: String,
      user_id: String,
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

    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = mcp_entity::Entity::find()
            .select_only()
            .columns([
              mcp_entity::Column::Id,
              mcp_entity::Column::UserId,
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
            .filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_entity::Column::UserId.eq(&user_id_owned))
            .into_model::<McpWithServer>()
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok(
            results
              .into_iter()
              .map(|r| McpWithServerEntity {
                id: r.id,
                user_id: r.user_id,
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
        })
      })
      .await
  }

  async fn update_mcp(&self, tenant_id: &str, row: &McpEntity) -> Result<McpEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
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
          // Verify tenant ownership before update
          let existing = mcp_entity::Entity::find_by_id(row.id.clone())
            .filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          if existing.is_none() {
            return Err(DbError::ItemNotFound {
              id: row.id.clone(),
              item_type: "mcp".to_string(),
            });
          }
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(McpEntity::from(model))
        })
      })
      .await
  }

  async fn delete_mcp(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          mcp_entity::Entity::delete_many()
            .filter(mcp_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_entity::Column::UserId.eq(&user_id))
            .filter(mcp_entity::Column::Id.eq(&id))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }
}

#[cfg(test)]
#[path = "test_mcp_instance_repository.rs"]
mod test_mcp_instance_repository;
