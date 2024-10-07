use crate::{DataService, DataServiceError, LocalDataService};
use objs::{test_utils::temp_bodhi_home, Alias, RemoteModel};
use rstest::fixture;
use std::path::PathBuf;
use tempfile::TempDir;

#[fixture]
pub fn test_data_service(temp_bodhi_home: TempDir) -> TestDataService {
  let inner = LocalDataService::new(temp_bodhi_home.path().join("bodhi"));
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

impl DataService for TestDataService {
  fn list_aliases(&self) -> Result<Vec<Alias>> {
    self.inner.list_aliases()
  }

  fn save_alias(&self, alias: &Alias) -> Result<PathBuf> {
    self.inner.save_alias(alias)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.inner.find_alias(alias)
  }

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>> {
    self.inner.list_remote_models()
  }

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    self.inner.find_remote_model(alias)
  }

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    self.inner.copy_alias(alias, new_alias)
  }

  fn delete_alias(&self, alias: &str) -> Result<()> {
    self.inner.delete_alias(alias)
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
