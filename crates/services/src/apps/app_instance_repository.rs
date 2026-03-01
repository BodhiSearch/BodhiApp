use super::app_instance_entity::{self, AppInstanceRow};
use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  DbError, DefaultDbService,
};
use crate::AppStatus;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait AppInstanceRepository: Send + Sync {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError>;
  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &AppStatus,
  ) -> Result<(), DbError>;
  async fn update_app_instance_status(
    &self,
    client_id: &str,
    status: &AppStatus,
  ) -> Result<(), DbError>;
  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError>;
}

#[async_trait::async_trait]
impl AppInstanceRepository for DefaultDbService {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError> {
    let rows = app_instance_entity::Entity::find()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    if rows.len() > 1 {
      return Err(DbError::MultipleAppInstance);
    }

    match rows.into_iter().next() {
      Some(model) => {
        let client_secret = decrypt_api_key(
          &self.encryption_key,
          &model.encrypted_client_secret,
          &model.salt_client_secret,
          &model.nonce_client_secret,
        )
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

        Ok(Some(AppInstanceRow {
          client_id: model.client_id,
          client_secret,
          app_status: model.app_status,
          created_at: model.created_at,
          updated_at: model.updated_at,
        }))
      }
      None => Ok(None),
    }
  }

  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &AppStatus,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let (encrypted_secret, salt_secret, nonce_secret) =
      encrypt_api_key(&self.encryption_key, client_secret)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let active = app_instance_entity::ActiveModel {
      client_id: Set(client_id.to_string()),
      encrypted_client_secret: Set(encrypted_secret),
      salt_client_secret: Set(salt_secret),
      nonce_client_secret: Set(nonce_secret),
      app_status: Set(status.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    app_instance_entity::Entity::insert(active)
      .on_conflict(
        OnConflict::column(app_instance_entity::Column::ClientId)
          .update_columns([
            app_instance_entity::Column::EncryptedClientSecret,
            app_instance_entity::Column::SaltClientSecret,
            app_instance_entity::Column::NonceClientSecret,
            app_instance_entity::Column::AppStatus,
            app_instance_entity::Column::UpdatedAt,
          ])
          .to_owned(),
      )
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn update_app_instance_status(
    &self,
    client_id: &str,
    status: &AppStatus,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    let existing = app_instance_entity::Entity::find_by_id(client_id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    if existing.is_none() {
      return Err(DbError::ItemNotFound {
        id: client_id.to_string(),
        item_type: "app_instance".to_string(),
      });
    }

    let active = app_instance_entity::ActiveModel {
      client_id: Set(client_id.to_string()),
      app_status: Set(status.clone()),
      updated_at: Set(now),
      ..Default::default()
    };

    app_instance_entity::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError> {
    app_instance_entity::Entity::delete_by_id(client_id.to_string())
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }
}
