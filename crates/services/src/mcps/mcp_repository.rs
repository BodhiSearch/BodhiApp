use crate::db::{encryption::decrypt_api_key, DbError, DefaultDbService};
use crate::mcps::{
  McpAuthConfigEntity, McpAuthConfigParamEntity, McpAuthParamEntity, McpAuthType, McpEntity,
  McpOAuthConfig, McpOAuthConfigDetailEntity, McpOAuthToken, McpOAuthTokenEntity,
  McpWithServerEntity,
};
use mcp_client::McpAuthParams;
use sea_orm::{
  sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder,
  QuerySelect, Set,
};

use super::mcp_auth_config_entity;
use super::mcp_auth_config_param_entity;
use super::mcp_auth_param_entity;
use super::mcp_entity;
use super::mcp_oauth_config_detail_entity::{self, McpOAuthConfigDetailView};
use super::mcp_oauth_token_entity::{self, McpOAuthTokenView};
use super::mcp_server_entity;

/// Unified repository trait combining all MCP instance and auth operations.
/// Replaces the former `McpInstanceRepository` and `McpAuthRepository` traits.
#[async_trait::async_trait]
pub trait McpRepository: Send + Sync {
  // ============================================================================
  // MCP Instance methods (formerly McpInstanceRepository)
  // ============================================================================

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

  // ============================================================================
  // MCP Auth Config methods (formerly McpAuthRepository)
  // ============================================================================

  async fn create_mcp_auth_config(
    &self,
    row: &McpAuthConfigEntity,
  ) -> Result<McpAuthConfigEntity, DbError>;

  async fn get_mcp_auth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthConfigEntity>, DbError>;

  async fn list_mcp_auth_configs_by_server(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigEntity>, DbError>;

  async fn delete_mcp_auth_config(&self, tenant_id: &str, id: &str) -> Result<(), DbError>;

  // ---- MCP Auth Config Params (key definitions) ----

  async fn create_mcp_auth_config_param(
    &self,
    row: &McpAuthConfigParamEntity,
  ) -> Result<McpAuthConfigParamEntity, DbError>;

  async fn list_mcp_auth_config_params(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Vec<McpAuthConfigParamEntity>, DbError>;

  // ---- MCP OAuth Config Details (1:1 with auth_configs for oauth type) ----

  async fn create_mcp_oauth_config_detail(
    &self,
    row: &McpOAuthConfigDetailEntity,
  ) -> Result<McpOAuthConfigDetailEntity, DbError>;

  async fn get_mcp_oauth_config_detail(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Option<McpOAuthConfig>, DbError>;

  /// Get (client_id, decrypted_client_secret) for an OAuth config detail.
  async fn get_decrypted_client_secret(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Option<(String, String)>, DbError>;

  // ---- MCP Auth Params (instance-level header/query auth) ----

  async fn create_mcp_auth_param(
    &self,
    row: &McpAuthParamEntity,
  ) -> Result<McpAuthParamEntity, DbError>;

  async fn list_mcp_auth_params(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Vec<McpAuthParamEntity>, DbError>;

  async fn delete_mcp_auth_params_by_mcp(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<(), DbError>;

  /// Decrypt all auth params for an MCP instance and return as McpAuthParams.
  async fn get_decrypted_auth_params(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Option<McpAuthParams>, DbError>;

  // ---- MCP OAuth Token operations ----

  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError>;

  async fn get_mcp_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError>;

  async fn get_latest_oauth_token_by_mcp(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError>;

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError>;

  async fn delete_mcp_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError>;

  async fn delete_oauth_tokens_by_mcp(&self, tenant_id: &str, mcp_id: &str) -> Result<(), DbError>;

  async fn delete_oauth_tokens_by_mcp_and_user(
    &self,
    tenant_id: &str,
    mcp_id: &str,
    user_id: &str,
  ) -> Result<(), DbError>;

  /// Get the decrypted refresh token for an OAuth token, if present.
  async fn get_decrypted_refresh_token(
    &self,
    tenant_id: &str,
    token_id: &str,
  ) -> Result<Option<String>, DbError>;

  /// Get the decrypted access token for an OAuth token by ID.
  async fn get_decrypted_oauth_access_token(
    &self,
    tenant_id: &str,
    token_id: &str,
  ) -> Result<Option<String>, DbError>;

  /// Link an OAuth token to an MCP instance (set mcp_id).
  /// Verifies user_id ownership before updating.
  async fn link_oauth_token_to_mcp(
    &self,
    tenant_id: &str,
    token_id: &str,
    user_id: &str,
    mcp_id: &str,
  ) -> Result<(), DbError>;

  // ============================================================================
  // Composite methods (atomic multi-table operations)
  // ============================================================================

  /// Atomically create an MCP instance with optional auth params and OAuth token link.
  /// All operations happen within a single transaction.
  async fn create_mcp_with_auth(
    &self,
    tenant_id: &str,
    row: &McpEntity,
    auth_params: Option<Vec<McpAuthParamEntity>>,
    oauth_token_id: Option<String>,
    user_id: &str,
  ) -> Result<McpEntity, DbError>;

  /// Atomically update an MCP instance with optional auth params and OAuth token link.
  /// All operations happen within a single transaction.
  async fn update_mcp_with_auth(
    &self,
    tenant_id: &str,
    row: &McpEntity,
    auth_params: Option<Vec<McpAuthParamEntity>>,
    oauth_token_id: Option<String>,
    user_id: &str,
  ) -> Result<McpEntity, DbError>;

  /// Atomically create a header auth config with its param entries.
  async fn create_auth_config_header(
    &self,
    tenant_id: &str,
    config_entity: &McpAuthConfigEntity,
    params: Vec<McpAuthConfigParamEntity>,
  ) -> Result<McpAuthConfigEntity, DbError>;

  /// Atomically create an OAuth auth config with its detail row.
  async fn create_auth_config_oauth(
    &self,
    tenant_id: &str,
    config_entity: &McpAuthConfigEntity,
    oauth_detail: &McpOAuthConfigDetailEntity,
  ) -> Result<(McpAuthConfigEntity, McpOAuthConfigDetailEntity), DbError>;

  /// Atomically store an OAuth token, deleting any existing tokens for (mcp_id, user_id) first.
  async fn store_oauth_token(
    &self,
    tenant_id: &str,
    mcp_id: Option<String>,
    user_id: &str,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError>;
}

#[async_trait::async_trait]
impl McpRepository for DefaultDbService {
  // ============================================================================
  // MCP Instance methods (formerly in mcp_instance_repository.rs)
  // ============================================================================

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
            auth_type: Set(row.auth_type.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
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
      auth_type: McpAuthType,
      auth_config_id: Option<String>,
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
              mcp_entity::Column::AuthType,
              mcp_entity::Column::AuthConfigId,
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
                auth_type: r.auth_type,
                auth_config_id: r.auth_config_id,
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
            auth_type: Set(row.auth_type.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
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

  // ============================================================================
  // MCP Auth Config methods (formerly in mcp_auth_repository.rs)
  // ============================================================================

  async fn create_mcp_auth_config(
    &self,
    row: &McpAuthConfigEntity,
  ) -> Result<McpAuthConfigEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_auth_config_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            mcp_server_id: Set(row.mcp_server_id.clone()),
            config_type: Set(row.config_type.clone()),
            name: Set(row.name.clone()),
            created_by: Set(row.created_by.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpAuthConfigEntity::from(model))
        })
      })
      .await
  }

  async fn get_mcp_auth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthConfigEntity>, DbError> {
    let id_owned = id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_auth_config_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_auth_config_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn list_mcp_auth_configs_by_server(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigEntity>, DbError> {
    let mcp_server_id_owned = mcp_server_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = mcp_auth_config_entity::Entity::find()
            .filter(mcp_auth_config_entity::Column::McpServerId.eq(&mcp_server_id_owned))
            .order_by(mcp_auth_config_entity::Column::CreatedAt, Order::Desc)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn delete_mcp_auth_config(&self, tenant_id: &str, id: &str) -> Result<(), DbError> {
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // NULL out auth_config_id references in mcps table (for SQLite FK compatibility)
          mcp_entity::Entity::update_many()
            .col_expr(
              mcp_entity::Column::AuthConfigId,
              Expr::value(sea_orm::Value::String(None)),
            )
            .filter(mcp_entity::Column::AuthConfigId.eq(&id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;

          // Delete the auth config
          mcp_auth_config_entity::Entity::delete_by_id(&id_owned)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  // ---- MCP Auth Config Params ----

  async fn create_mcp_auth_config_param(
    &self,
    row: &McpAuthConfigParamEntity,
  ) -> Result<McpAuthConfigParamEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_auth_config_param_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
            param_type: Set(row.param_type.clone()),
            param_key: Set(row.param_key.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpAuthConfigParamEntity::from(model))
        })
      })
      .await
  }

  async fn list_mcp_auth_config_params(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Vec<McpAuthConfigParamEntity>, DbError> {
    let auth_config_id_owned = auth_config_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = mcp_auth_config_param_entity::Entity::find()
            .filter(mcp_auth_config_param_entity::Column::AuthConfigId.eq(&auth_config_id_owned))
            .order_by(mcp_auth_config_param_entity::Column::CreatedAt, Order::Asc)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  // ---- MCP OAuth Config Details ----

  async fn create_mcp_oauth_config_detail(
    &self,
    row: &McpOAuthConfigDetailEntity,
  ) -> Result<McpOAuthConfigDetailEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_oauth_config_detail_entity::ActiveModel {
            auth_config_id: Set(row.auth_config_id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            registration_type: Set(row.registration_type.clone()),
            client_id: Set(row.client_id.clone()),
            encrypted_client_secret: Set(row.encrypted_client_secret.clone()),
            client_secret_salt: Set(row.client_secret_salt.clone()),
            client_secret_nonce: Set(row.client_secret_nonce.clone()),
            authorization_endpoint: Set(row.authorization_endpoint.clone()),
            token_endpoint: Set(row.token_endpoint.clone()),
            registration_endpoint: Set(row.registration_endpoint.clone()),
            encrypted_registration_access_token: Set(
              row.encrypted_registration_access_token.clone(),
            ),
            registration_access_token_salt: Set(row.registration_access_token_salt.clone()),
            registration_access_token_nonce: Set(row.registration_access_token_nonce.clone()),
            client_id_issued_at: Set(row.client_id_issued_at),
            token_endpoint_auth_method: Set(row.token_endpoint_auth_method.clone()),
            scopes: Set(row.scopes.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpOAuthConfigDetailEntity::from(model))
        })
      })
      .await
  }

  async fn get_mcp_oauth_config_detail(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Option<McpOAuthConfig>, DbError> {
    let id_owned = auth_config_id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let base = mcp_auth_config_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_auth_config_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          let base = match base {
            Some(b) => b,
            None => return Ok(None),
          };

          let detail = mcp_oauth_config_detail_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_oauth_config_detail_entity::Column::TenantId.eq(&tenant_id_owned))
            .into_partial_model::<McpOAuthConfigDetailView>()
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match detail {
            Some(v) => Ok(Some(McpOAuthConfig {
              id: base.id,
              name: base.name,
              mcp_server_id: base.mcp_server_id,
              registration_type: v.registration_type,
              client_id: v.client_id,
              authorization_endpoint: v.authorization_endpoint,
              token_endpoint: v.token_endpoint,
              registration_endpoint: v.registration_endpoint,
              client_id_issued_at: v.client_id_issued_at.map(|dt| dt.timestamp()),
              token_endpoint_auth_method: v.token_endpoint_auth_method,
              scopes: v.scopes,
              has_client_secret: v.encrypted_client_secret.is_some(),
              has_registration_access_token: v.encrypted_registration_access_token.is_some(),
              created_at: v.created_at,
              updated_at: v.updated_at,
            })),
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn get_decrypted_client_secret(
    &self,
    tenant_id: &str,
    auth_config_id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let id_owned = auth_config_id.to_string();
    let tenant_id_owned = tenant_id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_config_detail_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_oauth_config_detail_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(model) => {
              if let (Some(ref enc), Some(ref salt), Some(ref nonce)) = (
                &model.encrypted_client_secret,
                &model.client_secret_salt,
                &model.client_secret_nonce,
              ) {
                let secret = decrypt_api_key(&encryption_key, enc, salt, nonce)
                  .map_err(|e| DbError::EncryptionError(e.to_string()))?;
                Ok(Some((model.client_id.clone(), secret)))
              } else {
                Ok(None)
              }
            }
            None => Err(DbError::ItemNotFound {
              id: id_owned,
              item_type: "mcp_oauth_config_detail".to_string(),
            }),
          }
        })
      })
      .await
  }

  // ---- MCP Auth Params ----

  async fn create_mcp_auth_param(
    &self,
    row: &McpAuthParamEntity,
  ) -> Result<McpAuthParamEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_auth_param_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            mcp_id: Set(row.mcp_id.clone()),
            param_type: Set(row.param_type.clone()),
            param_key: Set(row.param_key.clone()),
            encrypted_value: Set(row.encrypted_value.clone()),
            value_salt: Set(row.value_salt.clone()),
            value_nonce: Set(row.value_nonce.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpAuthParamEntity::from(model))
        })
      })
      .await
  }

  async fn list_mcp_auth_params(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Vec<McpAuthParamEntity>, DbError> {
    let mcp_id_owned = mcp_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = mcp_auth_param_entity::Entity::find()
            .filter(mcp_auth_param_entity::Column::McpId.eq(&mcp_id_owned))
            .order_by(mcp_auth_param_entity::Column::CreatedAt, Order::Asc)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn delete_mcp_auth_params_by_mcp(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<(), DbError> {
    let mcp_id_owned = mcp_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          mcp_auth_param_entity::Entity::delete_many()
            .filter(mcp_auth_param_entity::Column::McpId.eq(&mcp_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_decrypted_auth_params(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Option<McpAuthParams>, DbError> {
    let mcp_id_owned = mcp_id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let params = mcp_auth_param_entity::Entity::find()
            .filter(mcp_auth_param_entity::Column::McpId.eq(&mcp_id_owned))
            .all(txn)
            .await
            .map_err(DbError::from)?;

          if params.is_empty() {
            return Ok(None);
          }

          let mut headers = Vec::new();
          let mut query_params = Vec::new();

          for param in params {
            let value = decrypt_api_key(
              &encryption_key,
              &param.encrypted_value,
              &param.value_salt,
              &param.value_nonce,
            )
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;

            match param.param_type.as_str() {
              "header" => headers.push((param.param_key, value)),
              "query" => query_params.push((param.param_key, value)),
              _ => {} // ignore unknown types
            }
          }

          Ok(Some(McpAuthParams {
            headers,
            query_params,
          }))
        })
      })
      .await
  }

  // ---- MCP OAuth Token operations ----

  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_oauth_token_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            mcp_id: Set(row.mcp_id.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
            user_id: Set(row.user_id.clone()),
            encrypted_access_token: Set(row.encrypted_access_token.clone()),
            access_token_salt: Set(row.access_token_salt.clone()),
            access_token_nonce: Set(row.access_token_nonce.clone()),
            encrypted_refresh_token: Set(row.encrypted_refresh_token.clone()),
            refresh_token_salt: Set(row.refresh_token_salt.clone()),
            refresh_token_nonce: Set(row.refresh_token_nonce.clone()),
            scopes_granted: Set(row.scopes_granted.clone()),
            expires_at: Set(row.expires_at),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpOAuthTokenEntity::from(model))
        })
      })
      .await
  }

  async fn get_mcp_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError> {
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_token_entity::Entity::find_by_id(&id_owned)
            .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
            .into_partial_model::<McpOAuthTokenView>()
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.map(Into::into))
        })
      })
      .await
  }

  async fn get_latest_oauth_token_by_mcp(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError> {
    let mcp_id_owned = mcp_id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_token_entity::Entity::find()
            .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_oauth_token_entity::Column::McpId.eq(&mcp_id_owned))
            .order_by(mcp_oauth_token_entity::Column::CreatedAt, Order::Desc)
            .into_partial_model::<McpOAuthTokenView>()
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.map(Into::into))
        })
      })
      .await
  }

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError> {
    let row = row.clone();
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let active = mcp_oauth_token_entity::ActiveModel {
            id: Set(row.id.clone()),
            mcp_id: Set(row.mcp_id.clone()),
            encrypted_access_token: Set(row.encrypted_access_token.clone()),
            access_token_salt: Set(row.access_token_salt.clone()),
            access_token_nonce: Set(row.access_token_nonce.clone()),
            encrypted_refresh_token: Set(row.encrypted_refresh_token.clone()),
            refresh_token_salt: Set(row.refresh_token_salt.clone()),
            refresh_token_nonce: Set(row.refresh_token_nonce.clone()),
            scopes_granted: Set(row.scopes_granted.clone()),
            expires_at: Set(row.expires_at),
            updated_at: Set(row.updated_at),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(McpOAuthTokenEntity::from(model))
        })
      })
      .await
  }

  async fn delete_mcp_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError> {
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          mcp_oauth_token_entity::Entity::delete_many()
            .filter(mcp_oauth_token_entity::Column::Id.eq(&id_owned))
            .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn delete_oauth_tokens_by_mcp(&self, tenant_id: &str, mcp_id: &str) -> Result<(), DbError> {
    let mcp_id_owned = mcp_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          mcp_oauth_token_entity::Entity::delete_many()
            .filter(mcp_oauth_token_entity::Column::McpId.eq(&mcp_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn delete_oauth_tokens_by_mcp_and_user(
    &self,
    tenant_id: &str,
    mcp_id: &str,
    user_id: &str,
  ) -> Result<(), DbError> {
    let mcp_id_owned = mcp_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          mcp_oauth_token_entity::Entity::delete_many()
            .filter(mcp_oauth_token_entity::Column::McpId.eq(&mcp_id_owned))
            .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_decrypted_refresh_token(
    &self,
    tenant_id: &str,
    token_id: &str,
  ) -> Result<Option<String>, DbError> {
    let token_id_owned = token_id.to_string();
    let tenant_id_owned = tenant_id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_token_entity::Entity::find_by_id(&token_id_owned)
            .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(model) => {
              if let (Some(ref enc), Some(ref salt), Some(ref nonce)) = (
                &model.encrypted_refresh_token,
                &model.refresh_token_salt,
                &model.refresh_token_nonce,
              ) {
                let token = decrypt_api_key(&encryption_key, enc, salt, nonce)
                  .map_err(|e| DbError::EncryptionError(e.to_string()))?;
                Ok(Some(token))
              } else {
                Ok(None)
              }
            }
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn get_decrypted_oauth_access_token(
    &self,
    tenant_id: &str,
    token_id: &str,
  ) -> Result<Option<String>, DbError> {
    let token_id_owned = token_id.to_string();
    let tenant_id_owned = tenant_id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_token_entity::Entity::find_by_id(&token_id_owned)
            .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(model) => {
              let token = decrypt_api_key(
                &encryption_key,
                &model.encrypted_access_token,
                &model.access_token_salt,
                &model.access_token_nonce,
              )
              .map_err(|e| DbError::EncryptionError(e.to_string()))?;
              Ok(Some(token))
            }
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn link_oauth_token_to_mcp(
    &self,
    tenant_id: &str,
    token_id: &str,
    user_id: &str,
    mcp_id: &str,
  ) -> Result<(), DbError> {
    let token_id_owned = token_id.to_string();
    let user_id_owned = user_id.to_string();
    let mcp_id_owned = mcp_id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = mcp_oauth_token_entity::Entity::find_by_id(&token_id_owned)
            .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(model) => {
              let active = mcp_oauth_token_entity::ActiveModel {
                id: Set(model.id),
                mcp_id: Set(Some(mcp_id_owned)),
                ..Default::default()
              };
              active.update(txn).await.map_err(DbError::from)?;
              Ok(())
            }
            None => Err(DbError::ItemNotFound {
              id: token_id_owned,
              item_type: "mcp_oauth_token".to_string(),
            }),
          }
        })
      })
      .await
  }

  // ============================================================================
  // Composite methods
  // ============================================================================

  async fn create_mcp_with_auth(
    &self,
    tenant_id: &str,
    row: &McpEntity,
    auth_params: Option<Vec<McpAuthParamEntity>>,
    oauth_token_id: Option<String>,
    user_id: &str,
  ) -> Result<McpEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();
    let auth_params_owned = auth_params;
    let oauth_token_id_owned = oauth_token_id;
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // 1. Insert MCP row
          let active = mcp_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(tenant_id_owned.clone()),
            user_id: Set(row.user_id.clone()),
            mcp_server_id: Set(row.mcp_server_id.clone()),
            name: Set(row.name.clone()),
            slug: Set(row.slug.clone()),
            description: Set(row.description.clone()),
            enabled: Set(row.enabled),
            auth_type: Set(row.auth_type.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          let mcp_id = model.id.clone();

          // 2. Insert auth params if provided
          if let Some(params) = auth_params_owned {
            for param in params {
              let active = mcp_auth_param_entity::ActiveModel {
                id: Set(param.id.clone()),
                tenant_id: Set(param.tenant_id.clone()),
                mcp_id: Set(param.mcp_id.clone()),
                param_type: Set(param.param_type.clone()),
                param_key: Set(param.param_key.clone()),
                encrypted_value: Set(param.encrypted_value.clone()),
                value_salt: Set(param.value_salt.clone()),
                value_nonce: Set(param.value_nonce.clone()),
                created_at: Set(param.created_at),
                updated_at: Set(param.updated_at),
              };
              active.insert(txn).await.map_err(DbError::from)?;
            }
          }

          // 3. Link OAuth token if provided
          if let Some(token_id) = oauth_token_id_owned {
            let token = mcp_oauth_token_entity::Entity::find_by_id(&token_id)
              .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
              .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
              .one(txn)
              .await
              .map_err(DbError::from)?;

            match token {
              Some(t) => {
                let active = mcp_oauth_token_entity::ActiveModel {
                  id: Set(t.id),
                  mcp_id: Set(Some(mcp_id)),
                  ..Default::default()
                };
                active.update(txn).await.map_err(DbError::from)?;
              }
              None => {
                return Err(DbError::ItemNotFound {
                  id: token_id,
                  item_type: "mcp_oauth_token".to_string(),
                });
              }
            }
          }

          Ok(McpEntity::from(model))
        })
      })
      .await
  }

  async fn update_mcp_with_auth(
    &self,
    tenant_id: &str,
    row: &McpEntity,
    auth_params: Option<Vec<McpAuthParamEntity>>,
    oauth_token_id: Option<String>,
    user_id: &str,
  ) -> Result<McpEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();
    let auth_params_owned = auth_params;
    let oauth_token_id_owned = oauth_token_id;
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // 1. Verify tenant ownership and update MCP row
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

          let active = mcp_entity::ActiveModel {
            id: Set(row.id.clone()),
            name: Set(row.name.clone()),
            slug: Set(row.slug.clone()),
            description: Set(row.description.clone()),
            enabled: Set(row.enabled),
            auth_type: Set(row.auth_type.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
            updated_at: Set(row.updated_at),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          let mcp_id = model.id.clone();

          // 2. Replace auth params if provided (delete old + insert new)
          if let Some(params) = auth_params_owned {
            // Delete existing params
            mcp_auth_param_entity::Entity::delete_many()
              .filter(mcp_auth_param_entity::Column::McpId.eq(&mcp_id))
              .exec(txn)
              .await
              .map_err(DbError::from)?;

            // Insert new params
            for param in params {
              let active = mcp_auth_param_entity::ActiveModel {
                id: Set(param.id.clone()),
                tenant_id: Set(param.tenant_id.clone()),
                mcp_id: Set(param.mcp_id.clone()),
                param_type: Set(param.param_type.clone()),
                param_key: Set(param.param_key.clone()),
                encrypted_value: Set(param.encrypted_value.clone()),
                value_salt: Set(param.value_salt.clone()),
                value_nonce: Set(param.value_nonce.clone()),
                created_at: Set(param.created_at),
                updated_at: Set(param.updated_at),
              };
              active.insert(txn).await.map_err(DbError::from)?;
            }
          }

          // 3. Handle OAuth token linking
          if let Some(token_id) = oauth_token_id_owned {
            // Delete old tokens for this MCP
            mcp_oauth_token_entity::Entity::delete_many()
              .filter(mcp_oauth_token_entity::Column::McpId.eq(&mcp_id))
              .exec(txn)
              .await
              .map_err(DbError::from)?;

            // Link new token
            let token = mcp_oauth_token_entity::Entity::find_by_id(&token_id)
              .filter(mcp_oauth_token_entity::Column::TenantId.eq(&tenant_id_owned))
              .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
              .one(txn)
              .await
              .map_err(DbError::from)?;

            match token {
              Some(t) => {
                let active = mcp_oauth_token_entity::ActiveModel {
                  id: Set(t.id),
                  mcp_id: Set(Some(mcp_id)),
                  ..Default::default()
                };
                active.update(txn).await.map_err(DbError::from)?;
              }
              None => {
                return Err(DbError::ItemNotFound {
                  id: token_id,
                  item_type: "mcp_oauth_token".to_string(),
                });
              }
            }
          }

          Ok(McpEntity::from(model))
        })
      })
      .await
  }

  async fn create_auth_config_header(
    &self,
    tenant_id: &str,
    config_entity: &McpAuthConfigEntity,
    params: Vec<McpAuthConfigParamEntity>,
  ) -> Result<McpAuthConfigEntity, DbError> {
    let config = config_entity.clone();
    let params_owned = params;

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // 1. Insert base auth config
          let active = mcp_auth_config_entity::ActiveModel {
            id: Set(config.id.clone()),
            tenant_id: Set(config.tenant_id.clone()),
            mcp_server_id: Set(config.mcp_server_id.clone()),
            config_type: Set(config.config_type.clone()),
            name: Set(config.name.clone()),
            created_by: Set(config.created_by.clone()),
            created_at: Set(config.created_at),
            updated_at: Set(config.updated_at),
          };
          let config_model = active.insert(txn).await.map_err(DbError::from)?;

          // 2. Insert all param rows
          for param in params_owned {
            let active = mcp_auth_config_param_entity::ActiveModel {
              id: Set(param.id.clone()),
              tenant_id: Set(param.tenant_id.clone()),
              auth_config_id: Set(param.auth_config_id.clone()),
              param_type: Set(param.param_type.clone()),
              param_key: Set(param.param_key.clone()),
              created_at: Set(param.created_at),
              updated_at: Set(param.updated_at),
            };
            active.insert(txn).await.map_err(DbError::from)?;
          }

          Ok(McpAuthConfigEntity::from(config_model))
        })
      })
      .await
  }

  async fn create_auth_config_oauth(
    &self,
    tenant_id: &str,
    config_entity: &McpAuthConfigEntity,
    oauth_detail: &McpOAuthConfigDetailEntity,
  ) -> Result<(McpAuthConfigEntity, McpOAuthConfigDetailEntity), DbError> {
    let config = config_entity.clone();
    let detail = oauth_detail.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // 1. Insert base auth config
          let config_active = mcp_auth_config_entity::ActiveModel {
            id: Set(config.id.clone()),
            tenant_id: Set(config.tenant_id.clone()),
            mcp_server_id: Set(config.mcp_server_id.clone()),
            config_type: Set(config.config_type.clone()),
            name: Set(config.name.clone()),
            created_by: Set(config.created_by.clone()),
            created_at: Set(config.created_at),
            updated_at: Set(config.updated_at),
          };
          let config_model = config_active.insert(txn).await.map_err(DbError::from)?;

          // 2. Insert OAuth detail row
          let detail_active = mcp_oauth_config_detail_entity::ActiveModel {
            auth_config_id: Set(detail.auth_config_id.clone()),
            tenant_id: Set(detail.tenant_id.clone()),
            registration_type: Set(detail.registration_type.clone()),
            client_id: Set(detail.client_id.clone()),
            encrypted_client_secret: Set(detail.encrypted_client_secret.clone()),
            client_secret_salt: Set(detail.client_secret_salt.clone()),
            client_secret_nonce: Set(detail.client_secret_nonce.clone()),
            authorization_endpoint: Set(detail.authorization_endpoint.clone()),
            token_endpoint: Set(detail.token_endpoint.clone()),
            registration_endpoint: Set(detail.registration_endpoint.clone()),
            encrypted_registration_access_token: Set(
              detail.encrypted_registration_access_token.clone(),
            ),
            registration_access_token_salt: Set(detail.registration_access_token_salt.clone()),
            registration_access_token_nonce: Set(detail.registration_access_token_nonce.clone()),
            client_id_issued_at: Set(detail.client_id_issued_at),
            token_endpoint_auth_method: Set(detail.token_endpoint_auth_method.clone()),
            scopes: Set(detail.scopes.clone()),
            created_at: Set(detail.created_at),
            updated_at: Set(detail.updated_at),
          };
          let detail_model = detail_active.insert(txn).await.map_err(DbError::from)?;

          Ok((
            McpAuthConfigEntity::from(config_model),
            McpOAuthConfigDetailEntity::from(detail_model),
          ))
        })
      })
      .await
  }

  async fn store_oauth_token(
    &self,
    tenant_id: &str,
    mcp_id: Option<String>,
    user_id: &str,
    row: &McpOAuthTokenEntity,
  ) -> Result<McpOAuthTokenEntity, DbError> {
    let row = row.clone();
    let mcp_id_owned = mcp_id;
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Delete existing tokens for (mcp_id, user_id) to prevent orphaned rows
          if let Some(ref mid) = mcp_id_owned {
            mcp_oauth_token_entity::Entity::delete_many()
              .filter(mcp_oauth_token_entity::Column::McpId.eq(mid))
              .filter(mcp_oauth_token_entity::Column::UserId.eq(&user_id_owned))
              .exec(txn)
              .await
              .map_err(DbError::from)?;
          }

          // Insert new token
          let active = mcp_oauth_token_entity::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(row.tenant_id.clone()),
            mcp_id: Set(row.mcp_id.clone()),
            auth_config_id: Set(row.auth_config_id.clone()),
            user_id: Set(row.user_id.clone()),
            encrypted_access_token: Set(row.encrypted_access_token.clone()),
            access_token_salt: Set(row.access_token_salt.clone()),
            access_token_nonce: Set(row.access_token_nonce.clone()),
            encrypted_refresh_token: Set(row.encrypted_refresh_token.clone()),
            refresh_token_salt: Set(row.refresh_token_salt.clone()),
            refresh_token_nonce: Set(row.refresh_token_nonce.clone()),
            scopes_granted: Set(row.scopes_granted.clone()),
            expires_at: Set(row.expires_at),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(McpOAuthTokenEntity::from(model))
        })
      })
      .await
  }
}

#[cfg(test)]
#[path = "test_mcp_instance_repository.rs"]
mod test_mcp_instance_repository;

#[cfg(test)]
#[path = "test_mcp_auth_repository.rs"]
mod test_mcp_auth_repository;
