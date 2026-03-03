use crate::db::DbService;
use crate::models::{Alias, DataService, DataServiceError, UserAlias, UserAliasRequest};
use std::sync::Arc;

#[derive(Debug)]
pub struct MultiTenantDataService {
  db_service: Arc<dyn DbService>,
}

impl MultiTenantDataService {
  pub fn new(db_service: Arc<dyn DbService>) -> Self {
    Self { db_service }
  }
}

type Result<T> = std::result::Result<T, DataServiceError>;

#[async_trait::async_trait]
impl DataService for MultiTenantDataService {
  async fn list_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<Alias>> {
    // Only API aliases in multi-tenant mode
    let api_aliases = self
      .db_service
      .list_api_model_aliases(tenant_id, user_id)
      .await?;
    Ok(api_aliases.into_iter().map(Alias::Api).collect())
  }

  async fn find_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<Alias> {
    // Only from API aliases using supports_model()
    let api_aliases = self
      .db_service
      .list_api_model_aliases(tenant_id, user_id)
      .await
      .ok()?;
    api_aliases
      .into_iter()
      .find(|a| a.supports_model(alias))
      .map(Alias::Api)
  }

  async fn find_user_alias(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _alias: &str,
  ) -> Option<UserAlias> {
    None
  }

  async fn get_user_alias_by_id(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _id: &str,
  ) -> Option<UserAlias> {
    None
  }

  async fn copy_alias(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _id: &str,
    _new_alias: &str,
  ) -> Result<UserAlias> {
    Err(DataServiceError::Unsupported)
  }

  async fn delete_alias(&self, _tenant_id: &str, _user_id: &str, _id: &str) -> Result<()> {
    Err(DataServiceError::Unsupported)
  }

  async fn create_alias_from_form(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _form: UserAliasRequest,
  ) -> Result<UserAlias> {
    Err(DataServiceError::Unsupported)
  }

  async fn update_alias_from_form(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _id: &str,
    _form: UserAliasRequest,
  ) -> Result<UserAlias> {
    Err(DataServiceError::Unsupported)
  }
}
