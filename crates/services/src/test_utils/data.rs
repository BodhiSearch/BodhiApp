use crate::{
  test_utils::{db::test_db_service, test_hf_service, TestDbService, TestHfService},
  DataService, DataServiceError, LocalDataService,
};
use async_trait::async_trait;
use objs::{test_utils::temp_bodhi_home, Alias, UserAlias};
use rstest::fixture;
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;

#[fixture]
#[awt]
pub async fn test_data_service(
  temp_bodhi_home: TempDir,
  test_hf_service: TestHfService,
  #[future] test_db_service: TestDbService,
) -> TestDataService {
  let inner = LocalDataService::new(
    temp_bodhi_home.path().join("bodhi"),
    Arc::new(test_hf_service),
    Arc::new(test_db_service),
  );
  TestDataService {
    temp_bodhi_home,
    inner,
  }
}

#[derive(Debug)]
pub struct TestDataService {
  pub temp_bodhi_home: TempDir,
  pub inner: LocalDataService,
}

impl TestDataService {
  pub fn bodhi_home(&self) -> PathBuf {
    self.temp_bodhi_home.path().join("bodhi")
  }
}

type Result<T> = std::result::Result<T, DataServiceError>;

#[async_trait]
impl DataService for TestDataService {
  async fn list_aliases(&self) -> Result<Vec<Alias>> {
    self.inner.list_aliases().await
  }

  fn save_alias(&self, alias: &UserAlias) -> Result<PathBuf> {
    self.inner.save_alias(alias)
  }

  async fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.inner.find_alias(alias).await
  }

  fn find_user_alias(&self, alias: &str) -> Option<UserAlias> {
    self.inner.find_user_alias(alias)
  }

  async fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    self.inner.copy_alias(alias, new_alias).await
  }

  async fn delete_alias(&self, alias: &str) -> Result<()> {
    self.inner.delete_alias(alias).await
  }

  fn alias_filename(&self, alias: &str) -> Result<PathBuf> {
    self.inner.alias_filename(alias)
  }

  fn find_file(&self, folder: Option<String>, filename: &str) -> Result<PathBuf> {
    self.inner.find_file(folder, filename)
  }

  fn read_file(&self, folder: Option<String>, filename: &str) -> Result<Vec<u8>> {
    self.inner.read_file(folder, filename)
  }

  fn write_file(&self, folder: Option<String>, filename: &str, contents: &[u8]) -> Result<()> {
    self.inner.write_file(folder, filename, contents)
  }
}
