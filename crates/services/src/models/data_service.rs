use crate::auth::AuthContextError;
use crate::models::{Alias, BuilderError, ModelValidationError, Repo, UserAlias, UserAliasRequest};
use crate::{
  db::{DbError, DbService},
  HubService, HubServiceError, SNAPSHOT_MAIN,
};
use async_trait::async_trait;
use errmeta::{AppError, ErrorType};
use std::{fmt::Debug, sync::Arc};
use tracing::debug;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DataServiceError {
  #[error("Model configuration '{0}' already exists.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AliasExists(String),
  #[error("Model configuration '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  AliasNotFound(String),
  #[error("operation not supported in current deployment mode")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Unsupported,
  #[error(transparent)]
  Auth(#[from] AuthContextError),
  #[error(transparent)]
  HubService(#[from] HubServiceError),
  #[error(transparent)]
  Db(#[from] DbError),
  #[error(transparent)]
  ModelValidation(#[from] ModelValidationError),
  #[error(transparent)]
  Builder(#[from] BuilderError),
}

type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait DataService: Send + Sync + std::fmt::Debug {
  async fn list_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<Alias>>;
  async fn find_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<Alias>;
  async fn find_user_alias(&self, tenant_id: &str, user_id: &str, alias: &str)
    -> Option<UserAlias>;
  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Option<UserAlias>;
  async fn copy_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    new_alias: &str,
  ) -> Result<UserAlias>;
  async fn delete_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<()>;
  async fn create_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias>;
  async fn update_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias>;
}

#[derive(Debug, Clone)]
pub struct LocalDataService {
  hub_service: Arc<dyn HubService>,
  db_service: Arc<dyn DbService>,
}

impl LocalDataService {
  pub fn new(hub_service: Arc<dyn HubService>, db_service: Arc<dyn DbService>) -> Self {
    Self {
      hub_service,
      db_service,
    }
  }
}

#[async_trait]
impl DataService for LocalDataService {
  async fn list_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<Alias>> {
    // Get user aliases from DB
    let user_aliases = self
      .db_service
      .list_user_aliases(tenant_id, user_id)
      .await
      .unwrap_or_default();
    let mut result: Vec<Alias> = user_aliases.into_iter().map(Alias::User).collect();

    let model_aliases = self.hub_service.list_model_aliases()?;
    let model_alias_variants: Vec<Alias> = model_aliases.into_iter().map(Alias::Model).collect();

    result.extend(model_alias_variants);

    // Add API aliases from database
    match self
      .db_service
      .list_api_model_aliases(tenant_id, user_id)
      .await
    {
      Ok(api_aliases) => {
        let api_alias_variants: Vec<Alias> = api_aliases.into_iter().map(Alias::Api).collect();
        result.extend(api_alias_variants);
      }
      Err(_) => {
        // Continue without API aliases if database is not available
        // This provides graceful degradation
      }
    }

    result.sort_by(|a, b| {
      let alias_a = match a {
        Alias::User(user) => &user.alias,
        Alias::Model(model) => &model.alias,
        Alias::Api(api) => &api.id,
      };
      let alias_b = match b {
        Alias::User(user) => &user.alias,
        Alias::Model(model) => &model.alias,
        Alias::Api(api) => &api.id,
      };
      alias_a.cmp(alias_b)
    });
    Ok(result)
  }

  async fn find_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<Alias> {
    // Priority 1: Check user aliases (from DB)
    if let Some(user_alias) = self.find_user_alias(tenant_id, user_id, alias).await {
      return Some(Alias::User(user_alias));
    }

    // Priority 2: Check model aliases (auto-discovered GGUF files)
    if let Ok(model_aliases) = self.hub_service.list_model_aliases() {
      if let Some(model) = model_aliases.into_iter().find(|m| m.alias == alias) {
        return Some(Alias::Model(model));
      }
    }

    // Priority 3: Check API aliases (from database) - with prefix-aware routing
    if let Ok(api_aliases) = self
      .db_service
      .list_api_model_aliases(tenant_id, user_id)
      .await
    {
      if let Some(api) = api_aliases
        .into_iter()
        .find(|api| api.supports_model(alias))
      {
        return Some(Alias::Api(api));
      }
    }
    None
  }

  async fn find_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Option<UserAlias> {
    self
      .db_service
      .get_user_alias_by_name(tenant_id, user_id, alias)
      .await
      .ok()
      .flatten()
  }

  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Option<UserAlias> {
    self
      .db_service
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await
      .ok()
      .flatten()
  }

  async fn copy_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    new_alias: &str,
  ) -> Result<UserAlias> {
    let user_alias = self
      .db_service
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| DataServiceError::AliasNotFound(id.to_string()))?;

    // Check if new alias name already exists
    if let Ok(Some(_)) = self
      .db_service
      .get_user_alias_by_name(tenant_id, user_id, new_alias)
      .await
    {
      return Err(DataServiceError::AliasExists(new_alias.to_string()));
    }

    let now = self.db_service.now();
    let new_user_alias = UserAlias {
      id: ulid::Ulid::new().to_string(),
      alias: new_alias.to_string(),
      repo: user_alias.repo.clone(),
      filename: user_alias.filename.clone(),
      snapshot: user_alias.snapshot.clone(),
      request_params: user_alias.request_params.clone(),
      context_params: user_alias.context_params.clone(),
      created_at: now,
      updated_at: now,
    };

    self
      .db_service
      .create_user_alias(tenant_id, user_id, &new_user_alias)
      .await?;
    Ok(new_user_alias)
  }

  async fn delete_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<()> {
    // Check if alias exists first
    let _alias = self
      .db_service
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| DataServiceError::AliasNotFound(id.to_string()))?;

    self
      .db_service
      .delete_user_alias(tenant_id, user_id, id)
      .await?;
    Ok(())
  }

  async fn create_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias> {
    let alias_name = form.alias;
    let repo = Repo::try_from(form.repo)?;

    // Check if alias already exists
    if self
      .find_user_alias(tenant_id, user_id, &alias_name)
      .await
      .is_some()
    {
      return Err(DataServiceError::AliasExists(alias_name));
    }

    // Verify file exists locally
    let file_exists =
      self
        .hub_service
        .local_file_exists(&repo, &form.filename, form.snapshot.clone())?;
    if !file_exists {
      return Err(DataServiceError::HubService(
        HubServiceError::FileNotFound {
          filename: form.filename.clone(),
          repo: repo.to_string(),
          snapshot: form
            .snapshot
            .clone()
            .unwrap_or_else(|| SNAPSHOT_MAIN.to_string()),
        },
      ));
    }
    let local_model_file =
      self
        .hub_service
        .find_local_file(&repo, &form.filename, form.snapshot)?;

    let now = self.db_service.now();
    let user_alias = crate::models::UserAliasBuilder::default()
      .alias(alias_name)
      .repo(repo)
      .filename(form.filename)
      .snapshot(local_model_file.snapshot)
      .request_params(form.request_params.unwrap_or_default())
      .context_params(form.context_params.unwrap_or_default())
      .build_with_time(now)?;

    self
      .db_service
      .create_user_alias(tenant_id, user_id, &user_alias)
      .await?;
    debug!("model alias: '{}' saved to database", user_alias.alias);
    Ok(user_alias)
  }

  async fn update_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias> {
    // Get existing alias to verify it exists
    let existing = self
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await
      .ok_or_else(|| DataServiceError::AliasNotFound(id.to_string()))?;

    // If alias name changed, check uniqueness within (tenant_id, user_id) scope
    if form.alias != existing.alias
      && self
        .find_user_alias(tenant_id, user_id, &form.alias)
        .await
        .is_some()
    {
      return Err(DataServiceError::AliasExists(form.alias));
    }

    let repo = Repo::try_from(form.repo)?;

    // Verify file exists locally
    let file_exists =
      self
        .hub_service
        .local_file_exists(&repo, &form.filename, form.snapshot.clone())?;
    if !file_exists {
      return Err(DataServiceError::HubService(
        HubServiceError::FileNotFound {
          filename: form.filename.clone(),
          repo: repo.to_string(),
          snapshot: form
            .snapshot
            .clone()
            .unwrap_or_else(|| SNAPSHOT_MAIN.to_string()),
        },
      ));
    }
    let local_model_file =
      self
        .hub_service
        .find_local_file(&repo, &form.filename, form.snapshot)?;

    let now = self.db_service.now();
    let updated_alias = UserAlias {
      id: existing.id.clone(),
      alias: form.alias,
      repo,
      filename: form.filename,
      snapshot: local_model_file.snapshot,
      request_params: form.request_params.unwrap_or_default(),
      context_params: form.context_params.unwrap_or_default().into(),
      created_at: existing.created_at,
      updated_at: now,
    };

    self
      .db_service
      .update_user_alias(tenant_id, user_id, id, &updated_alias)
      .await?;
    Ok(updated_alias)
  }
}

#[cfg(test)]
#[path = "test_data_service.rs"]
mod test_data_service;
