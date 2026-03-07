use super::tenant_entity::{self, TenantRow};
use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  DbError, DefaultDbService,
};
use crate::AppStatus;
#[cfg(any(test, feature = "test-utils"))]
use crate::Tenant;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

impl DefaultDbService {
  fn decrypt_tenant_row(&self, model: tenant_entity::Model) -> Result<TenantRow, DbError> {
    let client_secret = if let (Some(enc), Some(salt), Some(nonce)) = (
      model.encrypted_client_secret.as_deref(),
      model.salt_client_secret.as_deref(),
      model.nonce_client_secret.as_deref(),
    ) {
      decrypt_api_key(&self.encryption_key, enc, salt, nonce)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?
    } else {
      String::new()
    };

    Ok(TenantRow {
      id: model.id,
      client_id: model.client_id,
      client_secret,
      app_status: model.app_status,
      created_by: model.created_by,
      created_at: model.created_at,
      updated_at: model.updated_at,
    })
  }
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TenantRepository: Send + Sync {
  async fn get_tenant(&self) -> Result<Option<TenantRow>, DbError>;
  async fn get_tenant_by_id(&self, id: &str) -> Result<Option<TenantRow>, DbError>;
  async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<TenantRow>, DbError>;
  async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &AppStatus,
    created_by: Option<String>,
  ) -> Result<TenantRow, DbError>;
  async fn update_tenant_status(&self, client_id: &str, status: &AppStatus) -> Result<(), DbError>;
  async fn update_tenant_created_by(
    &self,
    client_id: &str,
    created_by: &str,
  ) -> Result<(), DbError>;
  async fn delete_tenant(&self, client_id: &str) -> Result<(), DbError>;

  /// Test-only: insert a tenant with a caller-specified ID (bypasses ULID auto-generation).
  #[cfg(any(test, feature = "test-utils"))]
  async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError>;
}

#[async_trait::async_trait]
impl TenantRepository for DefaultDbService {
  async fn get_tenant(&self) -> Result<Option<TenantRow>, DbError> {
    let rows = tenant_entity::Entity::find()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    if rows.len() > 1 {
      return Err(DbError::MultipleTenant);
    }

    match rows.into_iter().next() {
      Some(model) => Ok(Some(self.decrypt_tenant_row(model)?)),
      None => Ok(None),
    }
  }

  async fn get_tenant_by_id(&self, id: &str) -> Result<Option<TenantRow>, DbError> {
    let model = tenant_entity::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match model {
      Some(model) => Ok(Some(self.decrypt_tenant_row(model)?)),
      None => Ok(None),
    }
  }

  async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<TenantRow>, DbError> {
    let model = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match model {
      Some(model) => Ok(Some(self.decrypt_tenant_row(model)?)),
      None => Ok(None),
    }
  }

  async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &AppStatus,
    created_by: Option<String>,
  ) -> Result<TenantRow, DbError> {
    let id = ulid::Ulid::new().to_string();
    let now = self.time_service.utc_now();
    let (encrypted_secret, salt_secret, nonce_secret) =
      encrypt_api_key(&self.encryption_key, client_secret)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let active = tenant_entity::ActiveModel {
      id: Set(id.clone()),
      client_id: Set(client_id.to_string()),
      encrypted_client_secret: Set(Some(encrypted_secret)),
      salt_client_secret: Set(Some(salt_secret)),
      nonce_client_secret: Set(Some(nonce_secret)),
      app_status: Set(status.clone()),
      created_by: Set(created_by.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    tenant_entity::Entity::insert(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(TenantRow {
      id,
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
      app_status: status.clone(),
      created_by,
      created_at: now,
      updated_at: now,
    })
  }

  async fn update_tenant_status(&self, client_id: &str, status: &AppStatus) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    let existing = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    let existing = existing.ok_or_else(|| DbError::ItemNotFound {
      id: client_id.to_string(),
      item_type: "tenant".to_string(),
    })?;

    let active = tenant_entity::ActiveModel {
      id: Set(existing.id),
      app_status: Set(status.clone()),
      updated_at: Set(now),
      ..Default::default()
    };

    tenant_entity::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn update_tenant_created_by(
    &self,
    client_id: &str,
    created_by: &str,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    let existing = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    let existing = existing.ok_or_else(|| DbError::ItemNotFound {
      id: client_id.to_string(),
      item_type: "tenant".to_string(),
    })?;

    let active = tenant_entity::ActiveModel {
      id: Set(existing.id),
      created_by: Set(Some(created_by.to_string())),
      updated_at: Set(now),
      ..Default::default()
    };

    tenant_entity::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn delete_tenant(&self, client_id: &str) -> Result<(), DbError> {
    use sea_orm::QueryFilter;
    let result = tenant_entity::Entity::delete_many()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    if result.rows_affected == 0 {
      return Err(DbError::ItemNotFound {
        id: client_id.to_string(),
        item_type: "tenant".to_string(),
      });
    }
    Ok(())
  }

  #[cfg(any(test, feature = "test-utils"))]
  async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError> {
    // Upsert: return existing tenant if client_id already exists
    if let Some(existing) = self.get_tenant_by_client_id(&tenant.client_id).await? {
      return Ok(existing);
    }
    let now = self.time_service.utc_now();
    let (encrypted_secret, salt_secret, nonce_secret) =
      encrypt_api_key(&self.encryption_key, &tenant.client_secret)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let active = tenant_entity::ActiveModel {
      id: Set(tenant.id.clone()),
      client_id: Set(tenant.client_id.clone()),
      encrypted_client_secret: Set(Some(encrypted_secret)),
      salt_client_secret: Set(Some(salt_secret)),
      nonce_client_secret: Set(Some(nonce_secret)),
      app_status: Set(tenant.status.clone()),
      created_by: Set(tenant.created_by.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    tenant_entity::Entity::insert(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(TenantRow {
      id: tenant.id.clone(),
      client_id: tenant.client_id.clone(),
      client_secret: tenant.client_secret.clone(),
      app_status: tenant.status.clone(),
      created_by: tenant.created_by.clone(),
      created_at: now,
      updated_at: now,
    })
  }
}
