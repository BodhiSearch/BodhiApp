use crate::db::{
  encryption::decrypt_api_key,
  entities::{mcp, mcp_auth_header, mcp_oauth_config, mcp_oauth_token, mcp_server},
  DbError, DefaultDbService, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow, McpRepository,
  McpRow, McpServerRow, McpWithServerRow,
};
use mcp_auth_header::McpAuthHeaderView;
use mcp_oauth_config::McpOAuthConfigView;
use mcp_oauth_token::McpOAuthTokenView;
use objs::{McpAuthHeader, McpAuthType, McpOAuthConfig, McpOAuthToken};
use sea_orm::{
  sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder,
  QuerySelect, Set, TransactionTrait,
};

#[async_trait::async_trait]
impl McpRepository for DefaultDbService {
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    let active = mcp_server::ActiveModel {
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
    let active = mcp_server::ActiveModel {
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
    let result = mcp_server::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpServerRow::from))
  }

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError> {
    let result = mcp_server::Entity::find()
      .filter(mcp_server::Column::Url.eq(url))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpServerRow::from))
  }

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError> {
    let mut query = mcp_server::Entity::find();
    if let Some(e) = enabled {
      query = query.filter(mcp_server::Column::Enabled.eq(e));
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

    let result = mcp::Entity::find()
      .select_only()
      .column_as(
        Expr::cust("COALESCE(SUM(CASE WHEN enabled = true THEN 1 ELSE 0 END), 0)"),
        "enabled_count",
      )
      .column_as(
        Expr::cust("COALESCE(SUM(CASE WHEN enabled = false THEN 1 ELSE 0 END), 0)"),
        "disabled_count",
      )
      .filter(mcp::Column::McpServerId.eq(server_id))
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
    let result = mcp::Entity::update_many()
      .col_expr(
        mcp::Column::ToolsCache,
        Expr::value(sea_orm::Value::String(None)),
      )
      .col_expr(
        mcp::Column::ToolsFilter,
        Expr::value(sea_orm::Value::String(None)),
      )
      .col_expr(mcp::Column::UpdatedAt, Expr::value(now))
      .filter(mcp::Column::McpServerId.eq(server_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.rows_affected)
  }

  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    let active = mcp::ActiveModel {
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
    let result = mcp::Entity::find_by_id(id)
      .filter(mcp::Column::CreatedBy.eq(user_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(McpRow::from))
  }

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError> {
    let result = mcp::Entity::find()
      .filter(mcp::Column::CreatedBy.eq(user_id))
      .filter(mcp::Column::Slug.eq(slug))
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

    let results = mcp::Entity::find()
      .select_only()
      .columns([
        mcp::Column::Id,
        mcp::Column::CreatedBy,
        mcp::Column::McpServerId,
        mcp::Column::Name,
        mcp::Column::Slug,
        mcp::Column::Description,
        mcp::Column::Enabled,
        mcp::Column::ToolsCache,
        mcp::Column::ToolsFilter,
        mcp::Column::AuthType,
        mcp::Column::AuthUuid,
        mcp::Column::CreatedAt,
        mcp::Column::UpdatedAt,
      ])
      .column_as(mcp_server::Column::Url, "server_url")
      .column_as(mcp_server::Column::Name, "server_name")
      .column_as(mcp_server::Column::Enabled, "server_enabled")
      .join(
        sea_orm::JoinType::InnerJoin,
        mcp::Entity::belongs_to(mcp_server::Entity)
          .from(mcp::Column::McpServerId)
          .to(mcp_server::Column::Id)
          .into(),
      )
      .filter(mcp::Column::CreatedBy.eq(user_id))
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
    let active = mcp::ActiveModel {
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
    mcp::Entity::delete_many()
      .filter(mcp::Column::CreatedBy.eq(user_id))
      .filter(mcp::Column::Id.eq(id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    let active = mcp_auth_header::ActiveModel {
      id: Set(row.id.clone()),
      name: Set(row.name.clone()),
      mcp_server_id: Set(row.mcp_server_id.clone()),
      header_key: Set(row.header_key.clone()),
      encrypted_header_value: Set(row.encrypted_header_value.clone()),
      header_value_salt: Set(row.header_value_salt.clone()),
      header_value_nonce: Set(row.header_value_nonce.clone()),
      created_by: Set(row.created_by.clone()),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(McpAuthHeaderRow::from(model))
  }

  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<McpAuthHeader>, DbError> {
    let result = mcp_auth_header::Entity::find_by_id(id)
      .into_partial_model::<McpAuthHeaderView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(Into::into))
  }

  async fn update_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    let active = mcp_auth_header::ActiveModel {
      id: Set(row.id.clone()),
      name: Set(row.name.clone()),
      header_key: Set(row.header_key.clone()),
      encrypted_header_value: Set(row.encrypted_header_value.clone()),
      header_value_salt: Set(row.header_value_salt.clone()),
      header_value_nonce: Set(row.header_value_nonce.clone()),
      updated_at: Set(row.updated_at),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(McpAuthHeaderRow::from(model))
  }

  async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError> {
    mcp_auth_header::Entity::delete_by_id(id)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeader>, DbError> {
    let results = mcp_auth_header::Entity::find()
      .filter(mcp_auth_header::Column::McpServerId.eq(mcp_server_id))
      .order_by(mcp_auth_header::Column::CreatedAt, Order::Desc)
      .into_partial_model::<McpAuthHeaderView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError> {
    let result = mcp_auth_header::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(model) => {
        let value = decrypt_api_key(
          &self.encryption_key,
          &model.encrypted_header_value,
          &model.header_value_salt,
          &model.header_value_nonce,
        )
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some((model.header_key, value)))
      }
      None => Ok(None),
    }
  }

  async fn create_mcp_oauth_config(
    &self,
    row: &McpOAuthConfigRow,
  ) -> Result<McpOAuthConfigRow, DbError> {
    let active = mcp_oauth_config::ActiveModel {
      id: Set(row.id.clone()),
      name: Set(row.name.clone()),
      mcp_server_id: Set(row.mcp_server_id.clone()),
      registration_type: Set(row.registration_type.clone()),
      client_id: Set(row.client_id.clone()),
      encrypted_client_secret: Set(row.encrypted_client_secret.clone()),
      client_secret_salt: Set(row.client_secret_salt.clone()),
      client_secret_nonce: Set(row.client_secret_nonce.clone()),
      authorization_endpoint: Set(row.authorization_endpoint.clone()),
      token_endpoint: Set(row.token_endpoint.clone()),
      registration_endpoint: Set(row.registration_endpoint.clone()),
      encrypted_registration_access_token: Set(row.encrypted_registration_access_token.clone()),
      registration_access_token_salt: Set(row.registration_access_token_salt.clone()),
      registration_access_token_nonce: Set(row.registration_access_token_nonce.clone()),
      client_id_issued_at: Set(row.client_id_issued_at),
      token_endpoint_auth_method: Set(row.token_endpoint_auth_method.clone()),
      scopes: Set(row.scopes.clone()),
      created_by: Set(row.created_by.clone()),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(McpOAuthConfigRow::from(model))
  }

  async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, DbError> {
    let result = mcp_oauth_config::Entity::find_by_id(id)
      .into_partial_model::<McpOAuthConfigView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(Into::into))
  }

  async fn list_mcp_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpOAuthConfig>, DbError> {
    let results = mcp_oauth_config::Entity::find()
      .filter(mcp_oauth_config::Column::McpServerId.eq(mcp_server_id))
      .order_by(mcp_oauth_config::Column::CreatedAt, Order::Desc)
      .into_partial_model::<McpOAuthConfigView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError> {
    mcp_oauth_config::Entity::delete_by_id(id)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError> {
    let txn = self.db.begin().await.map_err(DbError::from)?;

    mcp_oauth_token::Entity::delete_many()
      .filter(mcp_oauth_token::Column::McpOauthConfigId.eq(config_id))
      .exec(&txn)
      .await
      .map_err(DbError::from)?;

    mcp_oauth_config::Entity::delete_by_id(config_id)
      .exec(&txn)
      .await
      .map_err(DbError::from)?;

    txn.commit().await.map_err(DbError::from)?;
    Ok(())
  }

  async fn get_decrypted_client_secret(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let result = mcp_oauth_config::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(model) => {
        if let (Some(ref enc), Some(ref salt), Some(ref nonce)) = (
          &model.encrypted_client_secret,
          &model.client_secret_salt,
          &model.client_secret_nonce,
        ) {
          let secret = decrypt_api_key(&self.encryption_key, enc, salt, nonce)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          Ok(Some((model.client_id.clone(), secret)))
        } else {
          Ok(None)
        }
      }
      None => Err(DbError::ItemNotFound {
        id: id.to_string(),
        item_type: "mcp_oauth_config".to_string(),
      }),
    }
  }

  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    let active = mcp_oauth_token::ActiveModel {
      id: Set(row.id.clone()),
      mcp_oauth_config_id: Set(row.mcp_oauth_config_id.clone()),
      encrypted_access_token: Set(row.encrypted_access_token.clone()),
      access_token_salt: Set(row.access_token_salt.clone()),
      access_token_nonce: Set(row.access_token_nonce.clone()),
      encrypted_refresh_token: Set(row.encrypted_refresh_token.clone()),
      refresh_token_salt: Set(row.refresh_token_salt.clone()),
      refresh_token_nonce: Set(row.refresh_token_nonce.clone()),
      scopes_granted: Set(row.scopes_granted.clone()),
      expires_at: Set(row.expires_at),
      created_by: Set(row.created_by.clone()),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(McpOAuthTokenRow::from(model))
  }

  async fn get_mcp_oauth_token(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError> {
    let result = mcp_oauth_token::Entity::find_by_id(id)
      .filter(mcp_oauth_token::Column::CreatedBy.eq(user_id))
      .into_partial_model::<McpOAuthTokenView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(Into::into))
  }

  async fn get_latest_oauth_token_by_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError> {
    let result = mcp_oauth_token::Entity::find()
      .filter(mcp_oauth_token::Column::McpOauthConfigId.eq(config_id))
      .order_by(mcp_oauth_token::Column::CreatedAt, Order::Desc)
      .into_partial_model::<McpOAuthTokenView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(Into::into))
  }

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    let active = mcp_oauth_token::ActiveModel {
      id: Set(row.id.clone()),
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
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(McpOAuthTokenRow::from(model))
  }

  async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    mcp_oauth_token::Entity::delete_many()
      .filter(mcp_oauth_token::Column::Id.eq(id))
      .filter(mcp_oauth_token::Column::CreatedBy.eq(user_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError> {
    mcp_oauth_token::Entity::delete_many()
      .filter(mcp_oauth_token::Column::McpOauthConfigId.eq(config_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError> {
    mcp_oauth_token::Entity::delete_many()
      .filter(mcp_oauth_token::Column::McpOauthConfigId.eq(config_id))
      .filter(mcp_oauth_token::Column::CreatedBy.eq(user_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_decrypted_oauth_bearer(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let result = mcp_oauth_token::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(model) => {
        let token = decrypt_api_key(
          &self.encryption_key,
          &model.encrypted_access_token,
          &model.access_token_salt,
          &model.access_token_nonce,
        )
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some((
          "Authorization".to_string(),
          format!("Bearer {}", token),
        )))
      }
      None => Ok(None),
    }
  }

  async fn get_decrypted_refresh_token(&self, token_id: &str) -> Result<Option<String>, DbError> {
    let result = mcp_oauth_token::Entity::find_by_id(token_id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(model) => {
        if let (Some(ref enc), Some(ref salt), Some(ref nonce)) = (
          &model.encrypted_refresh_token,
          &model.refresh_token_salt,
          &model.refresh_token_nonce,
        ) {
          let token = decrypt_api_key(&self.encryption_key, enc, salt, nonce)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          Ok(Some(token))
        } else {
          Ok(None)
        }
      }
      None => Ok(None),
    }
  }
}
