use super::{temp_bodhi_home, temp_hf_home, MockDbService, MockEnvWrapper};
use crate::{
  db::{DbService, SqliteDbService},
  service::{
    AppService, AppServiceFn, AuthService, DataService, EnvService, EnvServiceFn, HfHubService,
    HubService, KeycloakAuthService, LocalDataService, MockAuthService, MockDataService,
    MockEnvServiceFn, MockHubService,
  },
};
use derive_builder::Builder;
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

#[allow(dead_code)]
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
  let service = AppService::new(
    Arc::new(env_service),
    Arc::new(hub_service),
    Arc::new(data_service),
    Arc::new(KeycloakAuthService::default()),
    Arc::new(SqliteDbService::no_op()),
  );
  AppServiceTuple(temp_bodhi_home, temp_hf_home, bodhi_home, hf_cache, service)
}

#[derive(Default, Builder)]
#[builder(default, setter(into))]
pub struct AppServiceStubMock {
  pub env_service: Arc<MockEnvServiceFn>,
  pub hub_service: Arc<MockHubService>,
  pub data_service: Arc<MockDataService>,
  pub auth_service: Arc<MockAuthService>,
  pub db_service: Arc<MockDbService>,
}

impl std::fmt::Debug for AppServiceStubMock {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "AppServiceStubMock")
  }
}

impl AppServiceStubMock {
  pub fn builder() -> AppServiceStubMockBuilder {
    AppServiceStubMockBuilder::default()
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

  fn auth_service(&self) -> Arc<dyn AuthService> {
    self.auth_service.clone()
  }

  fn db_service(&self) -> Arc<dyn DbService> {
    self.db_service.clone()
  }
}

#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  pub env_service: Option<Arc<dyn EnvServiceFn + Send + Sync>>,
  pub hub_service: Option<Arc<dyn HubService + Send + Sync>>,
  pub data_service: Option<Arc<dyn DataService + Send + Sync>>,
  pub auth_service: Option<Arc<dyn AuthService + Send + Sync>>,
  pub db_service: Option<Arc<dyn DbService + Send + Sync>>,
}

impl AppServiceFn for AppServiceStub {
  fn env_service(&self) -> Arc<dyn EnvServiceFn> {
    self.env_service.clone().unwrap()
  }

  fn data_service(&self) -> Arc<dyn DataService> {
    self.data_service.clone().unwrap()
  }

  fn hub_service(&self) -> Arc<dyn HubService> {
    self.hub_service.clone().unwrap()
  }

  fn auth_service(&self) -> Arc<dyn AuthService> {
    self.auth_service.clone().unwrap()
  }

  fn db_service(&self) -> Arc<dyn DbService> {
    self.db_service.clone().unwrap()
  }
}
