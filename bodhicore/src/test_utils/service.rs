use super::{temp_bodhi_home, temp_hf_home, MockEnvWrapper};
use crate::service::{
  AppService, AppServiceBuilder, AppServiceFn, AuthService, DataService, EnvService, EnvServiceFn,
  HfHubService, HubService, KeycloakAuthService, LocalDataService, MockAuthService,
  MockDataService, MockEnvServiceFn, MockHubService,
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
  let auth_service = KeycloakAuthService::default();
  let service = AppServiceBuilder::default()
    .env_service(Arc::new(env_service))
    .hub_service(Arc::new(hub_service))
    .data_service(Arc::new(data_service))
    .auth_service(Arc::new(auth_service))
    .build()
    .unwrap();
  AppServiceTuple(temp_bodhi_home, temp_hf_home, bodhi_home, hf_cache, service)
}

use derive_builder::Builder;

#[derive(Debug, Default, Builder)]
#[builder(default)]
pub struct AppServiceStubMock {
  #[builder(setter(into))]
  pub env_service: Arc<MockEnvServiceFn>,
  #[builder(setter(into))]
  pub hub_service: Arc<MockHubService>,
  #[builder(setter(into))]
  pub data_service: Arc<MockDataService>,
  #[builder(setter(into))]
  pub auth_service: Arc<MockAuthService>,
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
}
