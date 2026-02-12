use crate::{
  test_utils::{
    db::test_db_service, seed_test_user_aliases, test_hf_service, TestDbService, TestHfService,
  },
  DataService, DataServiceError, LocalDataService,
};
use async_trait::async_trait;
use objs::{Alias, UserAlias};
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
  async fn list_aliases(&self) -> Result<Vec<Alias>> {
    self.inner.list_aliases().await
  }

  async fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.inner.find_alias(alias).await
  }

  async fn find_user_alias(&self, alias: &str) -> Option<UserAlias> {
    self.inner.find_user_alias(alias).await
  }

  async fn get_user_alias_by_id(&self, id: &str) -> Option<UserAlias> {
    self.inner.get_user_alias_by_id(id).await
  }

  async fn save_alias(&self, alias: &UserAlias) -> Result<()> {
    self.inner.save_alias(alias).await
  }

  async fn copy_alias(&self, id: &str, new_alias: &str) -> Result<UserAlias> {
    self.inner.copy_alias(id, new_alias).await
  }

  async fn delete_alias(&self, id: &str) -> Result<()> {
    self.inner.delete_alias(id).await
  }
}
