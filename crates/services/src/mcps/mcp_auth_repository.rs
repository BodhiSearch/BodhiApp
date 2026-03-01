use crate::db::{encryption::decrypt_api_key, DbError, DefaultDbService};
use crate::mcps::{
  McpAuthHeader, McpAuthHeaderRow, McpOAuthConfig, McpOAuthConfigRow, McpOAuthToken,
  McpOAuthTokenRow,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use super::mcp_auth_header_entity::{self, McpAuthHeaderView};
use super::mcp_oauth_config_entity::{self, McpOAuthConfigView};
use super::mcp_oauth_token_entity::{self, McpOAuthTokenView};

#[async_trait::async_trait]
pub trait McpAuthRepository: Send + Sync {
  // MCP auth header configs
  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError>;

  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<McpAuthHeader>, DbError>;

  async fn update_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError>;

  async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError>;

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeader>, DbError>;

  /// Get the decrypted auth header (key, value) for an MCP auth header config.
  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError>;

  // MCP OAuth config operations
  async fn create_mcp_oauth_config(
    &self,
    row: &McpOAuthConfigRow,
  ) -> Result<McpOAuthConfigRow, DbError>;

  async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, DbError>;

  async fn list_mcp_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpOAuthConfig>, DbError>;

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError>;

  /// Delete an OAuth config and all its associated tokens in a single transaction.
  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError>;

  /// Get (client_id, decrypted_client_secret) for an OAuth config.
  async fn get_decrypted_client_secret(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError>;

  // MCP OAuth token operations
  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError>;

  async fn get_mcp_oauth_token(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError>;

  async fn get_latest_oauth_token_by_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpOAuthToken>, DbError>;

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError>;

  async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError>;

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError>;

  /// Delete existing tokens for a specific (config_id, user_id) pair.
  /// Used before inserting a new token to prevent orphaned rows.
  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError>;

  /// Get decrypted OAuth bearer header (Authorization, Bearer <token>) by token ID.
  /// Not user-scoped; used for admin preview flows.
  async fn get_decrypted_oauth_bearer(&self, id: &str)
    -> Result<Option<(String, String)>, DbError>;

  /// Get the decrypted refresh token for an OAuth token, if present.
  async fn get_decrypted_refresh_token(&self, token_id: &str) -> Result<Option<String>, DbError>;
}

#[async_trait::async_trait]
impl McpAuthRepository for DefaultDbService {
  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    let active = mcp_auth_header_entity::ActiveModel {
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
    let result = mcp_auth_header_entity::Entity::find_by_id(id)
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
    let active = mcp_auth_header_entity::ActiveModel {
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
    mcp_auth_header_entity::Entity::delete_by_id(id)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeader>, DbError> {
    let results = mcp_auth_header_entity::Entity::find()
      .filter(mcp_auth_header_entity::Column::McpServerId.eq(mcp_server_id))
      .order_by(mcp_auth_header_entity::Column::CreatedAt, Order::Desc)
      .into_partial_model::<McpAuthHeaderView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError> {
    let result = mcp_auth_header_entity::Entity::find_by_id(id)
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
    let active = mcp_oauth_config_entity::ActiveModel {
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
    let result = mcp_oauth_config_entity::Entity::find_by_id(id)
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
    let results = mcp_oauth_config_entity::Entity::find()
      .filter(mcp_oauth_config_entity::Column::McpServerId.eq(mcp_server_id))
      .order_by(mcp_oauth_config_entity::Column::CreatedAt, Order::Desc)
      .into_partial_model::<McpOAuthConfigView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError> {
    mcp_oauth_config_entity::Entity::delete_by_id(id)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError> {
    let txn = self.db.begin().await.map_err(DbError::from)?;

    mcp_oauth_token_entity::Entity::delete_many()
      .filter(mcp_oauth_token_entity::Column::McpOauthConfigId.eq(config_id))
      .exec(&txn)
      .await
      .map_err(DbError::from)?;

    mcp_oauth_config_entity::Entity::delete_by_id(config_id)
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
    let result = mcp_oauth_config_entity::Entity::find_by_id(id)
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
    let active = mcp_oauth_token_entity::ActiveModel {
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
    let result = mcp_oauth_token_entity::Entity::find_by_id(id)
      .filter(mcp_oauth_token_entity::Column::CreatedBy.eq(user_id))
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
    let result = mcp_oauth_token_entity::Entity::find()
      .filter(mcp_oauth_token_entity::Column::McpOauthConfigId.eq(config_id))
      .order_by(mcp_oauth_token_entity::Column::CreatedAt, Order::Desc)
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
    let active = mcp_oauth_token_entity::ActiveModel {
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
    mcp_oauth_token_entity::Entity::delete_many()
      .filter(mcp_oauth_token_entity::Column::Id.eq(id))
      .filter(mcp_oauth_token_entity::Column::CreatedBy.eq(user_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError> {
    mcp_oauth_token_entity::Entity::delete_many()
      .filter(mcp_oauth_token_entity::Column::McpOauthConfigId.eq(config_id))
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
    mcp_oauth_token_entity::Entity::delete_many()
      .filter(mcp_oauth_token_entity::Column::McpOauthConfigId.eq(config_id))
      .filter(mcp_oauth_token_entity::Column::CreatedBy.eq(user_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_decrypted_oauth_bearer(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let result = mcp_oauth_token_entity::Entity::find_by_id(id)
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
    let result = mcp_oauth_token_entity::Entity::find_by_id(token_id)
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

#[cfg(test)]
#[path = "test_mcp_auth_repository.rs"]
mod test_mcp_auth_repository;
