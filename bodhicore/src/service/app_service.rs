use super::{
  data_service::{DataService, LocalDataService},
  hub_service::{HfHubService, HubService},
  DataServiceError, HubServiceError,
};
use crate::objs::{Alias, HubFile, RemoteModel, Repo};
use std::{fmt::Debug, path::PathBuf, sync::Arc};

pub trait AppServiceFn: HubService + DataService + Send + Sync {}

#[derive(Debug, Clone)]
pub struct AppService {
  pub(super) hub_service: Arc<dyn HubService + Send + Sync>,
  pub(super) data_service: Arc<dyn DataService + Send + Sync>,
}

impl Default for AppService {
  fn default() -> Self {
    Self {
      hub_service: Arc::new(HfHubService::default()),
      data_service: Arc::new(LocalDataService::default()),
    }
  }
}

impl AppService {
  pub fn new(hub_service: HfHubService, data_service: LocalDataService) -> Self {
    Self {
      hub_service: Arc::new(hub_service),
      data_service: Arc::new(data_service),
    }
  }
}

impl HubService for AppService {
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

impl DataService for AppService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>, DataServiceError> {
    self.data_service.find_remote_model(alias)
  }

  fn save_alias(&self, alias: Alias) -> Result<PathBuf, DataServiceError> {
    self.data_service.save_alias(alias)
  }

  fn list_aliases(&self) -> Result<Vec<Alias>, DataServiceError> {
    self.data_service.list_aliases()
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.data_service.find_alias(alias)
  }

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>, DataServiceError> {
    self.data_service.list_remote_models()
  }
}

impl AppServiceFn for AppService {}
