use super::llm_liberty_credentials_entity::{self as creds_entity, LlmLibertyCredentialsView};
use super::llm_liberty_envelope::{
  LlmLibertyEnvelope, LlmLibertySummary, ResolvedLlmLibertyCredentials,
};
use crate::db::encryption::{decrypt_api_key, encrypt_api_key};
use crate::db::{DbError, DefaultDbService};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait LlmLibertyCredentialsRepository: Send + Sync {
  /// Persist credentials for a new llm_liberty_oauth alias.
  async fn create_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
    envelope: &LlmLibertyEnvelope,
  ) -> Result<(), DbError>;

  /// Replace all credentials atomically (re-paste flow).
  async fn update_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
    envelope: &LlmLibertyEnvelope,
  ) -> Result<(), DbError>;

  /// Update only the token columns after a successful refresh.
  /// `tenant_id` is required for the tenant-scoped transaction.
  async fn update_llm_liberty_tokens(
    &self,
    tenant_id: &str,
    api_alias_id: &str,
    new_access_token: &str,
    new_refresh_token: &str,
    new_expires_at: DateTime<Utc>,
  ) -> Result<(), DbError>;

  /// Read credentials; decrypts access_token and refresh_token.
  async fn get_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<Option<ResolvedLlmLibertyCredentials>, DbError>;

  /// Return the summary (no secrets) for ApiAliasResponse.
  async fn get_llm_liberty_summary(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<Option<LlmLibertySummary>, DbError>;

  /// Delete credentials row (FK cascade on alias delete; this is for explicit calls).
  async fn delete_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<(), DbError>;
}

#[async_trait]
impl LlmLibertyCredentialsRepository for DefaultDbService {
  async fn create_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
    envelope: &LlmLibertyEnvelope,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let (enc_access, access_salt, access_nonce) =
      encrypt_api_key(&self.encryption_key, &envelope.access_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
    let (enc_refresh, refresh_salt, refresh_nonce) =
      encrypt_api_key(&self.encryption_key, &envelope.refresh_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let expires_at = DateTime::from_timestamp(envelope.expires_at, 0).unwrap_or(now);

    let model = creds_entity::ActiveModel {
      api_alias_id: Set(api_alias_id.to_string()),
      tenant_id: Set(tenant_id.to_string()),
      user_id: Set(user_id.to_string()),
      envelope_version: Set(envelope.version.clone()),
      provider: Set(envelope.provider.clone()),
      encrypted_access_token: Set(enc_access),
      access_salt: Set(access_salt),
      access_nonce: Set(access_nonce),
      encrypted_refresh_token: Set(enc_refresh),
      refresh_salt: Set(refresh_salt),
      refresh_nonce: Set(refresh_nonce),
      expires_at: Set(expires_at),
      auth_in: Set(envelope.auth.location.clone()),
      auth_key: Set(envelope.auth.key.clone()),
      auth_scheme: Set(envelope.auth.scheme.clone()),
      oauth_authorize_url: Set(envelope.oauth.authorize_url.clone()),
      oauth_token_url: Set(envelope.oauth.token_url.clone()),
      oauth_revoke_url: Set(envelope.oauth.revoke_url.clone()),
      oauth_client_id: Set(envelope.oauth.client_id.clone()),
      oauth_client_secret: Set(envelope.oauth.client_secret.clone()),
      api_base_url: Set(envelope.api.base_url.clone()),
      api_chat_url: Set(envelope.api.chat_url.clone()),
      api_models_url: Set(envelope.api.models_url.clone()),
      headers_json: Set(envelope.headers.clone()),
      body_json: Set(envelope.body.clone()),
      extra_json: Set(envelope.extra.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          creds_entity::Entity::insert(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn update_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
    envelope: &LlmLibertyEnvelope,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let (enc_access, access_salt, access_nonce) =
      encrypt_api_key(&self.encryption_key, &envelope.access_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
    let (enc_refresh, refresh_salt, refresh_nonce) =
      encrypt_api_key(&self.encryption_key, &envelope.refresh_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let expires_at = DateTime::from_timestamp(envelope.expires_at, 0).unwrap_or(now);
    let api_alias_id_owned = api_alias_id.to_string();
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let envelope = envelope.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Verify ownership before updating; missing or cross-tenant row returns NotFound
          // so the caller can surface the failure (vs. a silent no-op).
          let owns = creds_entity::Entity::find_by_id(&api_alias_id_owned)
            .into_partial_model::<LlmLibertyCredentialsView>()
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(|v| v.tenant_id == tenant_id_owned && v.user_id == user_id_owned)
            .unwrap_or(false);

          if !owns {
            return Err(DbError::ItemNotFound {
              id: api_alias_id_owned,
              item_type: "llm_liberty_credentials".to_string(),
            });
          }

          let model = creds_entity::ActiveModel {
            api_alias_id: Set(api_alias_id_owned),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(user_id_owned),
            envelope_version: Set(envelope.version),
            provider: Set(envelope.provider),
            encrypted_access_token: Set(enc_access),
            access_salt: Set(access_salt),
            access_nonce: Set(access_nonce),
            encrypted_refresh_token: Set(enc_refresh),
            refresh_salt: Set(refresh_salt),
            refresh_nonce: Set(refresh_nonce),
            expires_at: Set(expires_at),
            auth_in: Set(envelope.auth.location),
            auth_key: Set(envelope.auth.key),
            auth_scheme: Set(envelope.auth.scheme),
            oauth_authorize_url: Set(envelope.oauth.authorize_url),
            oauth_token_url: Set(envelope.oauth.token_url),
            oauth_revoke_url: Set(envelope.oauth.revoke_url),
            oauth_client_id: Set(envelope.oauth.client_id),
            oauth_client_secret: Set(envelope.oauth.client_secret),
            api_base_url: Set(envelope.api.base_url),
            api_chat_url: Set(envelope.api.chat_url),
            api_models_url: Set(envelope.api.models_url),
            headers_json: Set(envelope.headers),
            body_json: Set(envelope.body),
            extra_json: Set(envelope.extra),
            updated_at: Set(now),
            created_at: sea_orm::ActiveValue::NotSet,
          };
          creds_entity::Entity::update(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn update_llm_liberty_tokens(
    &self,
    tenant_id: &str,
    api_alias_id: &str,
    new_access_token: &str,
    new_refresh_token: &str,
    new_expires_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    let (enc_access, access_salt, access_nonce) =
      encrypt_api_key(&self.encryption_key, new_access_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
    let (enc_refresh, refresh_salt, refresh_nonce) =
      encrypt_api_key(&self.encryption_key, new_refresh_token)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
    let now = self.time_service.utc_now();
    let api_alias_id_owned = api_alias_id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // SQLite has no RLS; filter explicitly so a wrong tenant_id can't rotate
          // another tenant's tokens via a known alias_id.
          let res = creds_entity::Entity::update_many()
            .col_expr(
              creds_entity::Column::EncryptedAccessToken,
              enc_access.into(),
            )
            .col_expr(creds_entity::Column::AccessSalt, access_salt.into())
            .col_expr(creds_entity::Column::AccessNonce, access_nonce.into())
            .col_expr(
              creds_entity::Column::EncryptedRefreshToken,
              enc_refresh.into(),
            )
            .col_expr(creds_entity::Column::RefreshSalt, refresh_salt.into())
            .col_expr(creds_entity::Column::RefreshNonce, refresh_nonce.into())
            .col_expr(creds_entity::Column::ExpiresAt, new_expires_at.into())
            .col_expr(creds_entity::Column::UpdatedAt, now.into())
            .filter(creds_entity::Column::ApiAliasId.eq(&api_alias_id_owned))
            .filter(creds_entity::Column::TenantId.eq(&tenant_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          if res.rows_affected == 0 {
            return Err(DbError::ItemNotFound {
              id: api_alias_id_owned,
              item_type: "llm_liberty_credentials".to_string(),
            });
          }
          Ok(())
        })
      })
      .await
  }

  async fn get_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<Option<ResolvedLlmLibertyCredentials>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let api_alias_id_owned = api_alias_id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let row = creds_entity::Entity::find_by_id(&api_alias_id_owned)
            .one(txn)
            .await
            .map_err(DbError::from)?;

          let Some(row) = row else {
            return Ok(None);
          };
          if row.tenant_id != tenant_id_owned || row.user_id != user_id_owned {
            return Ok(None);
          }

          let access_token = decrypt_api_key(
            &encryption_key,
            &row.encrypted_access_token,
            &row.access_salt,
            &row.access_nonce,
          )
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          let refresh_token = decrypt_api_key(
            &encryption_key,
            &row.encrypted_refresh_token,
            &row.refresh_salt,
            &row.refresh_nonce,
          )
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;

          Ok(Some(ResolvedLlmLibertyCredentials {
            access_token,
            refresh_token,
            expires_at: row.expires_at,
            tenant_id: row.tenant_id,
            provider: row.provider,
            auth_scheme: row.auth_scheme,
            auth_key: row.auth_key,
            oauth_token_url: row.oauth_token_url,
            oauth_client_id: row.oauth_client_id,
            oauth_client_secret: row.oauth_client_secret,
            api_base_url: row.api_base_url,
            api_chat_url: row.api_chat_url,
            api_models_url: row.api_models_url,
            headers_json: row.headers_json,
            body_json: row.body_json,
            extra_json: row.extra_json,
          }))
        })
      })
      .await
  }

  async fn get_llm_liberty_summary(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<Option<LlmLibertySummary>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let api_alias_id_owned = api_alias_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let row = creds_entity::Entity::find_by_id(&api_alias_id_owned)
            .into_partial_model::<LlmLibertyCredentialsView>()
            .one(txn)
            .await
            .map_err(DbError::from)?;

          let Some(row) = row else {
            return Ok(None);
          };
          if row.tenant_id != tenant_id_owned || row.user_id != user_id_owned {
            return Ok(None);
          }

          Ok(Some(LlmLibertySummary {
            provider: row.provider,
            envelope_version: row.envelope_version,
            expires_at: row.expires_at.timestamp(),
            has_refresh_token: true,
          }))
        })
      })
      .await
  }

  async fn delete_llm_liberty_credentials(
    &self,
    tenant_id: &str,
    user_id: &str,
    api_alias_id: &str,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let api_alias_id_owned = api_alias_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Verify ownership before deleting; missing or cross-tenant row returns NotFound.
          let owns = creds_entity::Entity::find_by_id(&api_alias_id_owned)
            .into_partial_model::<LlmLibertyCredentialsView>()
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(|v| v.tenant_id == tenant_id_owned && v.user_id == user_id_owned)
            .unwrap_or(false);

          if !owns {
            return Err(DbError::ItemNotFound {
              id: api_alias_id_owned,
              item_type: "llm_liberty_credentials".to_string(),
            });
          }

          creds_entity::Entity::delete_by_id(api_alias_id_owned)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }
}
