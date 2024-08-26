use super::{
  copy_test_dir, temp_bodhi_home, temp_hf_home, temp_home, MockDbService, MockEnvWrapper,
  SecretServiceStub,
};
use crate::{
  db::DbService,
  service::{
    AppServiceFn, AuthService, CacheService, DataService, EnvService, EnvServiceFn, HfHubService,
    HubService, ISecretService, LocalDataService, MockAuthService, MockCacheService,
    MockDataService, MockEnvServiceFn, MockHubService, MockISecretService, MockSessionService,
    MokaCacheService, SessionService,
  },
};
use derive_builder::Builder;
use rstest::fixture;
use std::{path::PathBuf, sync::Arc, time::Duration};
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

#[derive(Default, Builder)]
#[builder(default, setter(into))]
pub struct AppServiceStubMock {
  pub env_service: Arc<MockEnvServiceFn>,
  pub hub_service: Arc<MockHubService>,
  pub data_service: Arc<MockDataService>,
  pub auth_service: Arc<MockAuthService>,
  pub db_service: Arc<MockDbService>,
  pub session_service: Arc<MockSessionService>,
  pub secret_service: Arc<MockISecretService>,
  pub cache_service: Arc<MockCacheService>,
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

  fn session_service(&self) -> Arc<dyn SessionService> {
    self.session_service.clone()
  }

  fn secret_service(&self) -> Arc<dyn ISecretService> {
    self.secret_service.clone()
  }

  fn cache_service(&self) -> Arc<dyn CacheService> {
    self.cache_service.clone()
  }
}

#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  pub env_service: Option<Arc<dyn EnvServiceFn + Send + Sync>>,
  pub hub_service: Option<Arc<dyn HubService + Send + Sync>>,
  pub temp_home: Option<Arc<TempDir>>,
  pub data_service: Option<Arc<dyn DataService + Send + Sync>>,
  #[builder(default = "self.default_auth_service()")]
  pub auth_service: Option<Arc<dyn AuthService + Send + Sync>>,
  pub db_service: Option<Arc<dyn DbService + Send + Sync>>,
  pub session_service: Option<Arc<dyn SessionService + Send + Sync>>,
  #[builder(default = "self.default_secret_service()")]
  pub secret_service: Option<Arc<dyn ISecretService + Send + Sync>>,
  #[builder(default = "self.default_cache_service()")]
  pub cache_service: Option<Arc<dyn CacheService + Send + Sync>>,
}

impl AppServiceStubBuilder {
  fn default_cache_service(&self) -> Option<Arc<dyn CacheService + Send + Sync>> {
    Some(Arc::new(MokaCacheService::new(
      Some(100),
      Some(Duration::from_secs(30 * 24 * 60 * 60)),
    )))
  }

  fn default_auth_service(&self) -> Option<Arc<dyn AuthService + Send + Sync>> {
    Some(Arc::new(MockAuthService::default()))
  }

  fn default_secret_service(&self) -> Option<Arc<dyn ISecretService + Send + Sync>> {
    Some(Arc::new(SecretServiceStub::default()))
  }
}

impl AppServiceStub {
  pub fn bodhi_home(&self) -> PathBuf {
    self.temp_home.clone().unwrap().path().join("bodhi")
  }
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

  fn session_service(&self) -> Arc<dyn SessionService> {
    self.session_service.clone().unwrap()
  }

  fn secret_service(&self) -> Arc<dyn ISecretService> {
    self.secret_service.clone().unwrap()
  }

  fn cache_service(&self) -> Arc<dyn CacheService> {
    self.cache_service.clone().unwrap()
  }
}

impl AppServiceStubBuilder {
  pub fn with_hub_service(&mut self) -> &mut Self {
    let temp_home = self.with_temp_home();
    let hf_home = temp_home.path().join("huggingface");
    copy_test_dir("tests/data/huggingface", &hf_home);
    let hf_cache = hf_home.join("hub");
    let hub_service = HfHubService::new(hf_cache, false, None);
    self.hub_service = Some(Some(Arc::new(hub_service)));
    self
  }

  pub fn with_data_service(&mut self) -> &mut Self {
    let temp_home = self.with_temp_home();
    let bodhi_home = temp_home.path().join("bodhi");
    copy_test_dir("tests/data/bodhi", &bodhi_home);
    let data_service = LocalDataService::new(bodhi_home);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }

  pub fn with_temp_home(&mut self) -> Arc<TempDir> {
    match &self.temp_home {
      Some(Some(temp_home)) => temp_home.clone(),
      None | Some(None) => {
        let temp_home = Arc::new(temp_home());
        self.temp_home = Some(Some(temp_home.clone()));
        let env_service = EnvService::new_with_args(
          MockEnvWrapper::default(),
          temp_home.path().join("bodhi"),
          temp_home.path().join("huggingface"),
        );
        self.env_service = Some(Some(Arc::new(env_service)));
        temp_home
      }
    }
  }
}
