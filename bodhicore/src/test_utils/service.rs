use super::{temp_bodhi_home, temp_hf_home};
use crate::{
  objs::{Alias, HubFile, RemoteModel},
  service::{
    AppService, AppServiceFn, DataService, DataServiceError, HfHubService, HubService,
    HubServiceError, LocalDataService, MockDataService, MockHubService,
  },
  Repo,
};
use derive_new::new;
use rstest::fixture;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct HubServiceTuple(pub TempDir, pub PathBuf, pub HfHubService);

#[fixture]
pub fn hub_service(temp_hf_home: TempDir) -> HubServiceTuple {
  let hf_cache = temp_hf_home.path().join("huggingface/hub");
  let hub_service = HfHubService::new(hf_cache.clone(), false, None);
  HubServiceTuple(temp_hf_home, hf_cache, hub_service)
}

pub struct DataServiceTuple(pub TempDir, pub PathBuf, pub LocalDataService);

#[fixture]
pub fn data_service(temp_bodhi_home: TempDir) -> DataServiceTuple {
  let bodhi_home = temp_bodhi_home.path().join("bodhi");
  let data_service = LocalDataService::new(bodhi_home.clone());
  DataServiceTuple(temp_bodhi_home, bodhi_home, data_service)
}

pub struct AppServiceTuple(
  pub TempDir,
  pub TempDir,
  pub PathBuf,
  pub PathBuf,
  pub AppService,
);

#[fixture]
pub fn app_service_stub(
  hub_service: HubServiceTuple,
  data_service: DataServiceTuple,
) -> AppServiceTuple {
  let DataServiceTuple(temp_bodhi_home, bodhi_home, data_service) = data_service;
  let HubServiceTuple(temp_hf_home, hf_cache, hub_service) = hub_service;
  let service = AppService::new(hub_service, data_service);
  AppServiceTuple(temp_bodhi_home, temp_hf_home, bodhi_home, hf_cache, service)
}

#[derive(Debug, new)]
pub struct MockAppServiceFn {
  pub hub_service: MockHubService,
  pub data_service: MockDataService,
}

impl HubService for MockAppServiceFn {
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<HubFile, HubServiceError> {
    self.hub_service.download(repo, filename, force)
  }

  fn list_local_models(&self) -> Vec<HubFile> {
    self.hub_service.list_local_models()
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<HubFile>, HubServiceError> {
    self.hub_service.find_local_file(repo, filename, snapshot)
  }

  fn hf_home(&self) -> PathBuf {
    self.hub_service.hf_home()
  }

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf {
    self.hub_service.model_file_path(repo, filename, snapshot)
  }
}

impl DataService for MockAppServiceFn {
  fn bodhi_home(&self) -> PathBuf {
    self.data_service.bodhi_home()
  }

  fn list_aliases(&self) -> Result<Vec<Alias>, DataServiceError> {
    self.data_service.list_aliases()
  }

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>, DataServiceError> {
    self.data_service.find_remote_model(alias)
  }

  fn save_alias(&self, alias: Alias) -> Result<PathBuf, DataServiceError> {
    self.data_service.save_alias(alias)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.data_service.find_alias(alias)
  }

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>, DataServiceError> {
    self.data_service.list_remote_models()
  }
}

// Implement AppServiceFn for the combined struct
impl AppServiceFn for MockAppServiceFn {}

mockall::mock! {
  pub AppService {}

  impl std::fmt::Debug for AppService {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }

  unsafe impl Send for AppService { }

  unsafe impl Sync for AppService { }

  impl HubService for AppService {
    fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<HubFile, HubServiceError>;

    fn list_local_models(&self) -> Vec<HubFile>;

    fn find_local_file(
      &self,
      repo: &Repo,
      filename: &str,
      snapshot: &str,
    ) -> Result<Option<HubFile>, HubServiceError>;

    fn hf_home(&self) -> PathBuf;

    fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf;
  }

  impl DataService for AppService {
    fn bodhi_home(&self) -> PathBuf;

    fn list_aliases(&self) -> Result<Vec<Alias>, DataServiceError>;

    fn save_alias(&self, alias: Alias) -> Result<PathBuf, DataServiceError>;

    fn find_alias(&self, alias: &str) -> Option<Alias>;

    fn list_remote_models(&self) -> Result<Vec<RemoteModel>, DataServiceError>;

    fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>, DataServiceError>;
  }

  impl AppServiceFn for AppService { }
}
