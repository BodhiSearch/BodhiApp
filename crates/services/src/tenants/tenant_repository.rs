use super::tenant_entity::{self, TenantRow};
use super::tenant_user_entity;
use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  DbCore, DbError, DefaultDbService,
};
use crate::AppStatus;
#[cfg(any(test, feature = "test-utils"))]
use crate::Tenant;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

impl DefaultDbService {
  /// Upsert a tenant-user membership within an existing transaction.
  async fn upsert_tenant_user_on_txn(
    txn: &impl ConnectionTrait,
    tenant_id: &str,
    user_id: &str,
    now: DateTime<Utc>,
  ) -> Result<(), DbError> {
    let active = tenant_user_entity::ActiveModel {
      tenant_id: Set(tenant_id.to_string()),
      user_id: Set(user_id.to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    tenant_user_entity::Entity::insert(active)
      .on_conflict(
        OnConflict::columns([
          tenant_user_entity::Column::TenantId,
          tenant_user_entity::Column::UserId,
        ])
        .update_column(tenant_user_entity::Column::UpdatedAt)
        .to_owned(),
      )
      .exec(txn)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }
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
      name: model.name,
      description: model.description,
      app_status: model.app_status,
      created_by: model.created_by,
      created_at: model.created_at,
      updated_at: model.updated_at,
    })
  }

  /// Core tenant creation logic shared by `create_tenant` and `create_tenant_test`.
  /// Validates status/created_by, encrypts secret, inserts tenant row, and atomically
  /// upserts tenant-user membership when `created_by` is set.
  async fn create_tenant_impl(
    &self,
    id: &str,
    client_id: &str,
    client_secret: &str,
    name: &str,
    description: Option<String>,
    status: &AppStatus,
    created_by: Option<String>,
  ) -> Result<TenantRow, DbError> {
    if *status == AppStatus::Ready && created_by.is_none() {
      return Err(DbError::ValidationError(
        "created_by is required when status is Ready".to_string(),
      ));
    }

    let now = self.time_service.utc_now();
    let (encrypted_secret, salt_secret, nonce_secret) =
      encrypt_api_key(&self.encryption_key, client_secret)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let active = tenant_entity::ActiveModel {
      id: Set(id.to_string()),
      client_id: Set(client_id.to_string()),
      encrypted_client_secret: Set(Some(encrypted_secret)),
      salt_client_secret: Set(Some(salt_secret)),
      nonce_client_secret: Set(Some(nonce_secret)),
      name: Set(name.to_string()),
      description: Set(description.clone()),
      app_status: Set(status.clone()),
      created_by: Set(created_by.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };

    if let Some(ref user_id) = created_by {
      // Atomic: tenant insert + membership upsert in one transaction
      let txn = self.begin_tenant_txn(id).await?;
      tenant_entity::Entity::insert(active)
        .exec(&txn)
        .await
        .map_err(DbError::from)?;
      Self::upsert_tenant_user_on_txn(&txn, id, user_id, now).await?;
      txn.commit().await.map_err(DbError::from)?;
    } else {
      // Single insert, no transaction needed
      tenant_entity::Entity::insert(active)
        .exec(&self.db)
        .await
        .map_err(DbError::from)?;
    }

    Ok(TenantRow {
      id: id.to_string(),
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
      name: name.to_string(),
      description,
      app_status: status.clone(),
      created_by,
      created_at: now,
      updated_at: now,
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
    name: &str,
    description: Option<String>,
    status: &AppStatus,
    created_by: Option<String>,
  ) -> Result<TenantRow, DbError>;
  /// Atomically set tenant status to Ready, set created_by, and upsert tenant-user membership.
  async fn set_tenant_ready(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError>;
  async fn delete_tenant(&self, client_id: &str) -> Result<(), DbError>;

  /// Upsert a tenant-user membership. INSERT ON CONFLICT updates `updated_at`.
  async fn upsert_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError>;

  /// Delete a tenant-user membership (idempotent).
  async fn delete_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError>;

  /// List all tenants a user has membership in.
  async fn list_user_tenants(&self, user_id: &str) -> Result<Vec<TenantRow>, DbError>;

  /// Check if a user has any tenant memberships.
  async fn has_tenant_memberships(&self, user_id: &str) -> Result<bool, DbError>;

  /// Delete a tenant by its client_id, including associated tenant_users records.
  /// Idempotent: returns Ok if tenant does not exist.
  async fn delete_tenant_by_client_id(&self, client_id: &str) -> Result<(), DbError>;

  /// List all tenants created by a specific user (by `created_by` field).
  async fn list_tenants_by_creator(&self, created_by: &str) -> Result<Vec<TenantRow>, DbError>;

  /// Test-only: insert a tenant with a caller-specified ID (bypasses ULID auto-generation).
  /// Upsert-safe: returns existing tenant if client_id already exists.
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
    name: &str,
    description: Option<String>,
    status: &AppStatus,
    created_by: Option<String>,
  ) -> Result<TenantRow, DbError> {
    // One-per-user enforcement: in multi-tenant mode (created_by is Some),
    // check if the user already owns a tenant.
    if let Some(ref user_id) = created_by {
      let existing = self.list_tenants_by_creator(user_id).await?;
      if !existing.is_empty() {
        return Err(DbError::ValidationError(
          "user_already_has_tenant".to_string(),
        ));
      }
    }

    let id = crate::new_ulid();
    self
      .create_tenant_impl(
        &id,
        client_id,
        client_secret,
        name,
        description,
        status,
        created_by,
      )
      .await
  }

  async fn set_tenant_ready(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    let txn = self.begin_tenant_txn(tenant_id).await?;

    // Update tenant status + created_by in a single UPDATE by PK
    let existing = tenant_entity::Entity::find_by_id(tenant_id.to_string())
      .one(&txn)
      .await
      .map_err(DbError::from)?;

    let existing = existing.ok_or_else(|| DbError::ItemNotFound {
      id: tenant_id.to_string(),
      item_type: "tenant".to_string(),
    })?;

    let active = tenant_entity::ActiveModel {
      id: Set(existing.id),
      app_status: Set(AppStatus::Ready),
      created_by: Set(Some(user_id.to_string())),
      updated_at: Set(now),
      ..Default::default()
    };

    tenant_entity::Entity::update(active)
      .exec(&txn)
      .await
      .map_err(DbError::from)?;

    // Upsert tenant-user membership in the same transaction
    Self::upsert_tenant_user_on_txn(&txn, tenant_id, user_id, now).await?;

    txn.commit().await.map_err(DbError::from)?;

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

  async fn upsert_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          Self::upsert_tenant_user_on_txn(txn, &tenant_id_owned, &user_id_owned, now).await
        })
      })
      .await
  }

  async fn delete_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          tenant_user_entity::Entity::delete_many()
            .filter(tenant_user_entity::Column::TenantId.eq(&*tenant_id_owned))
            .filter(tenant_user_entity::Column::UserId.eq(&*user_id_owned))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn list_user_tenants(&self, user_id: &str) -> Result<Vec<TenantRow>, DbError> {
    // Two-step: get tenant_ids from membership, then fetch tenants
    let memberships = tenant_user_entity::Entity::find()
      .filter(tenant_user_entity::Column::UserId.eq(user_id))
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    let tenant_ids: Vec<String> = memberships.into_iter().map(|m| m.tenant_id).collect();
    if tenant_ids.is_empty() {
      return Ok(vec![]);
    }

    let models = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::Id.is_in(tenant_ids))
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    models
      .into_iter()
      .map(|m| self.decrypt_tenant_row(m))
      .collect()
  }

  async fn has_tenant_memberships(&self, user_id: &str) -> Result<bool, DbError> {
    let result = tenant_user_entity::Entity::find()
      .filter(tenant_user_entity::Column::UserId.eq(user_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.is_some())
  }

  async fn delete_tenant_by_client_id(&self, client_id: &str) -> Result<(), DbError> {
    let tenant = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    let tenant = match tenant {
      Some(t) => t,
      None => return Ok(()), // Idempotent: no tenant means nothing to delete
    };

    // Delete associated tenant_users first
    tenant_user_entity::Entity::delete_many()
      .filter(tenant_user_entity::Column::TenantId.eq(&tenant.id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    // Delete the tenant
    tenant_entity::Entity::delete_many()
      .filter(tenant_entity::Column::ClientId.eq(client_id))
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn list_tenants_by_creator(&self, created_by: &str) -> Result<Vec<TenantRow>, DbError> {
    let models = tenant_entity::Entity::find()
      .filter(tenant_entity::Column::CreatedBy.eq(created_by))
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    models
      .into_iter()
      .map(|m| self.decrypt_tenant_row(m))
      .collect()
  }

  #[cfg(any(test, feature = "test-utils"))]
  async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError> {
    // Upsert guard: return existing tenant if client_id already exists
    if let Some(existing) = self.get_tenant_by_client_id(&tenant.client_id).await? {
      return Ok(existing);
    }
    self
      .create_tenant_impl(
        &tenant.id,
        &tenant.client_id,
        &tenant.client_secret,
        &tenant.name,
        tenant.description.clone(),
        &tenant.status,
        tenant.created_by.clone(),
      )
      .await
  }
}
