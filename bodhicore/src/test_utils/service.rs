use super::{temp_bodhi_home, temp_hf_home, MockEnvWrapper};
use crate::service::{
  AppService, AppServiceFn, DataService, EnvService, EnvServiceFn, HfHubService, HubService,
  LocalDataService, MockDataService, MockEnvServiceFn, MockHubService,
};
use rstest::fixture;
use std::{path::PathBuf, sync::Arc};
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
  let mock = MockEnvWrapper::default();
  let env_service = EnvService::new_with_args(mock, bodhi_home.clone(), hf_cache.join(".."));
  let service = AppService::new(Arc::new(env_service), hub_service, data_service);
  AppServiceTuple(temp_bodhi_home, temp_hf_home, bodhi_home, hf_cache, service)
}

#[derive(Debug)]
pub struct AppServiceStubMock {
  pub env_service: Arc<MockEnvServiceFn>,
  pub hub_service: Arc<MockHubService>,
  pub data_service: Arc<MockDataService>,
}

impl AppServiceStubMock {
  pub fn new(
    env_service: MockEnvServiceFn,
    hub_service: MockHubService,
    data_service: MockDataService,
  ) -> Self {
    Self {
      env_service: Arc::new(env_service),
      hub_service: Arc::new(hub_service),
      data_service: Arc::new(data_service),
    }
  }
}

// Implement AppServiceFn for the combined struct
impl AppServiceFn for AppServiceStubMock {
  fn env_service(&self) -> Arc<dyn EnvServiceFn> {
    self.env_service.clone()
  }

  fn data_service(&self) -> Arc<dyn DataService> {
    self.data_service.clone()
  }

  fn hub_service(&self) -> Arc<dyn HubService> {
    self.hub_service.clone()
  }
}
