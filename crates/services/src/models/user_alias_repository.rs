use crate::db::{DbError, DefaultDbService};
use crate::models::user_alias_entity as user_alias;
use crate::models::UserAlias;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

#[async_trait::async_trait]
pub trait UserAliasRepository: Send + Sync {
  async fn create_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError>;
  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<UserAlias>, DbError>;
  async fn get_user_alias_by_name(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Result<Option<UserAlias>, DbError>;
  async fn update_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError>;
  async fn delete_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError>;
  async fn list_user_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<UserAlias>, DbError>;
}

#[async_trait::async_trait]
impl UserAliasRepository for DefaultDbService {
  async fn create_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let alias = alias.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let active = user_alias::ActiveModel {
            id: Set(alias.id.clone()),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(user_id),
            alias: Set(alias.alias.clone()),
            repo: Set(alias.repo.to_string()),
            filename: Set(alias.filename.clone()),
            snapshot: Set(alias.snapshot.clone()),
            request_params: Set(alias.request_params.clone()),
            context_params: Set(alias.context_params.clone()),
            created_at: Set(alias.created_at),
            updated_at: Set(alias.updated_at),
          };
          user_alias::Entity::insert(active)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<UserAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = user_alias::Entity::find_by_id(&id_owned)
            .filter(user_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(user_alias::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          match result {
            Some(model) => Ok(Some(UserAlias::try_from(model).map_err(|e| {
              DbError::Conversion(format!("Failed to convert user_alias: {}", e))
            })?)),
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn get_user_alias_by_name(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias_name: &str,
  ) -> Result<Option<UserAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let alias_name_owned = alias_name.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = user_alias::Entity::find()
            .filter(user_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(user_alias::Column::UserId.eq(&user_id_owned))
            .filter(user_alias::Column::Alias.eq(&alias_name_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          match result {
            Some(model) => Ok(Some(UserAlias::try_from(model).map_err(|e| {
              DbError::Conversion(format!("Failed to convert user_alias: {}", e))
            })?)),
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn update_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();
    let alias = alias.clone();
    let now = self.time_service.utc_now();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Verify ownership before updating
          let exists = user_alias::Entity::find()
            .filter(user_alias::Column::Id.eq(&id))
            .filter(user_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(user_alias::Column::UserId.eq(&user_id))
            .count(txn)
            .await
            .map_err(DbError::from)?
            > 0;

          if exists {
            let active = user_alias::ActiveModel {
              id: Set(id),
              alias: Set(alias.alias.clone()),
              repo: Set(alias.repo.to_string()),
              filename: Set(alias.filename.clone()),
              snapshot: Set(alias.snapshot.clone()),
              request_params: Set(alias.request_params.clone()),
              context_params: Set(alias.context_params.clone()),
              updated_at: Set(now),
              ..Default::default()
            };
            use sea_orm::ActiveModelTrait;
            active.update(txn).await.map_err(DbError::from)?;
          }
          Ok(())
        })
      })
      .await
  }

  async fn delete_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          user_alias::Entity::delete_many()
            .filter(user_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(user_alias::Column::UserId.eq(&user_id))
            .filter(user_alias::Column::Id.eq(&id))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn list_user_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<UserAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = user_alias::Entity::find()
            .filter(user_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(user_alias::Column::UserId.eq(&user_id_owned))
            .order_by_asc(user_alias::Column::Alias)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          results
            .into_iter()
            .map(|m| {
              UserAlias::try_from(m)
                .map_err(|e| DbError::Conversion(format!("Failed to convert user_alias: {}", e)))
            })
            .collect()
        })
      })
      .await
  }
}
