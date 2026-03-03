use crate::models::{Alias, UserAlias, UserAliasRequest};
use crate::{
  test_utils::{
    db::test_db_service, seed_test_user_aliases, test_hf_service, TestDbService, TestHfService,
  },
  DataService, DataServiceError, LocalDataService,
};
use async_trait::async_trait;
use rstest::fixture;
use std::sync::Arc;

#[fixture]
#[awt]
pub async fn test_data_service(
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> TestDataService {
  let db_service = Arc::new(test_db_service);
  // Seed user aliases into DB
  seed_test_user_aliases(db_service.as_ref()).await.unwrap();
  let inner = LocalDataService::new(Arc::new(test_hf_service), db_service);
  TestDataService { inner }
}

#[derive(Debug)]
pub struct TestDataService {
  pub inner: LocalDataService,
}

type Result<T> = std::result::Result<T, DataServiceError>;

#[async_trait]
impl DataService for TestDataService {
  async fn list_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<Alias>> {
    self.inner.list_aliases(tenant_id, user_id).await
  }

  async fn find_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<Alias> {
    self.inner.find_alias(tenant_id, user_id, alias).await
  }

  async fn find_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Option<UserAlias> {
    self.inner.find_user_alias(tenant_id, user_id, alias).await
  }

  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Option<UserAlias> {
    self
      .inner
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await
  }

  async fn copy_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    new_alias: &str,
  ) -> Result<UserAlias> {
    self
      .inner
      .copy_alias(tenant_id, user_id, id, new_alias)
      .await
  }

  async fn delete_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<()> {
    self.inner.delete_alias(tenant_id, user_id, id).await
  }

  async fn create_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias> {
    self
      .inner
      .create_alias_from_form(tenant_id, user_id, form)
      .await
  }

  async fn update_alias_from_form(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias> {
    self
      .inner
      .update_alias_from_form(tenant_id, user_id, id, form)
      .await
  }
}
