use super::error::ModelRouterError;
use crate::db::{DbService, TimeService};
use crate::models::{
  Alias, ModelRouterAlias, ModelRouterRequest, ModelRouterResponse, RouterTarget,
};
use crate::{new_ulid, DataService};
use async_trait::async_trait;
use errmeta::EntityError;
use std::sync::Arc;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait ModelRouterService: Send + Sync + std::fmt::Debug {
  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError>;

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError>;

  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), ModelRouterError>;

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ModelRouterResponse, ModelRouterError>;
}

#[derive(Debug, derive_new::new)]
pub struct DefaultModelRouterService {
  db_service: Arc<dyn DbService>,
  data_service: Arc<dyn DataService>,
  time_service: Arc<dyn TimeService>,
}

impl DefaultModelRouterService {
  /// Business validation shared by create and update. A zero-target or all-disabled
  /// router is allowed (empty active set errors at request time, not at save time).
  async fn validate(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias_name: &str,
    targets: &[RouterTarget],
    exclude_id: Option<&str>,
  ) -> Result<(), ModelRouterError> {
    // Name must be unique across all alias kinds. Router-vs-router uses the dedicated
    // existence check (covers a not-yet-listed sibling); other kinds are checked by
    // identity (`alias_name`) against the aggregate list, excluding self on update.
    let router_clash = self
      .db_service
      .check_router_alias_exists(
        tenant_id,
        user_id,
        alias_name,
        exclude_id.map(str::to_string),
      )
      .await?;
    if router_clash {
      return Err(ModelRouterError::AliasExists {
        alias: alias_name.to_string(),
      });
    }

    // Aliases are referenced by identity (`alias_name`): name for user/model/router,
    // id for api. Resolve targets and cross-kind name collisions from this one list.
    let aliases = self
      .data_service
      .list_aliases(tenant_id, user_id)
      .await
      .unwrap_or_default();

    if aliases
      .iter()
      .any(|a| !a.is_model_router() && a.alias_name() == alias_name)
    {
      return Err(ModelRouterError::AliasExists {
        alias: alias_name.to_string(),
      });
    }

    // Validate each declared target (enabled or not).
    for target in targets {
      if target.alias == alias_name {
        return Err(ModelRouterError::SelfReference {
          alias: target.alias.clone(),
        });
      }
      let inner = match aliases.iter().find(|a| a.alias_name() == target.alias) {
        Some(a) if a.is_model_router() => {
          return Err(ModelRouterError::NestedRouterNotAllowed {
            alias: target.alias.clone(),
          })
        }
        Some(a) => a,
        None => {
          return Err(ModelRouterError::ReferencedAliasNotFound {
            alias: target.alias.clone(),
          })
        }
      };
      if let Alias::Api(api) = inner {
        if !api.api_format.supports_chat_completions() {
          return Err(ModelRouterError::TargetFormatUnsupported {
            alias: target.alias.clone(),
            api_format: api.api_format.to_string(),
          });
        }
      }
      if !inner.can_serve(&target.model) {
        return Err(ModelRouterError::InvalidPinnedModel {
          alias: target.alias.clone(),
          model: target.model.clone(),
        });
      }
    }
    Ok(())
  }
}

#[async_trait]
impl ModelRouterService for DefaultModelRouterService {
  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError> {
    let targets: Vec<RouterTarget> = form.targets.into_iter().map(Into::into).collect();
    self
      .validate(tenant_id, user_id, &form.alias, &targets, None)
      .await?;

    let now = self.time_service.utc_now();
    let alias = ModelRouterAlias {
      id: new_ulid(),
      alias: form.alias,
      targets,
      strategy: form.strategy,
      created_at: now,
      updated_at: now,
    };
    self
      .db_service
      .create_model_router_alias(tenant_id, user_id, &alias)
      .await?;
    Ok(alias.into())
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError> {
    let targets: Vec<RouterTarget> = form.targets.into_iter().map(Into::into).collect();
    self
      .validate(tenant_id, user_id, &form.alias, &targets, Some(id))
      .await?;

    let now = self.time_service.utc_now();
    let alias = ModelRouterAlias {
      id: id.to_string(),
      alias: form.alias,
      targets,
      strategy: form.strategy,
      created_at: now,
      updated_at: now,
    };
    self
      .db_service
      .update_model_router_alias(tenant_id, user_id, id, &alias)
      .await?;
    self.get(tenant_id, user_id, id).await
  }

  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), ModelRouterError> {
    self
      .db_service
      .delete_model_router_alias(tenant_id, user_id, id)
      .await?;
    Ok(())
  }

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ModelRouterResponse, ModelRouterError> {
    let alias = self
      .db_service
      .get_model_router_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("model-router '{}' not found", id)))?;
    Ok(alias.into())
  }
}
