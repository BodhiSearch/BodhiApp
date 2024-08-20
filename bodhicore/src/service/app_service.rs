use super::{
  data_service::{DataService, LocalDataService},
  hub_service::{HfHubService, HubService},
  AuthService, EnvServiceFn, KeycloakAuthService,
};
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait AppServiceFn: std::fmt::Debug + Send + Sync {
  fn env_service(&self) -> Arc<dyn EnvServiceFn>;

  fn data_service(&self) -> Arc<dyn DataService>;

  fn hub_service(&self) -> Arc<dyn HubService>;

  fn auth_service(&self) -> Arc<dyn AuthService>;
}

#[derive(Clone, Debug)]
pub struct AppService {
  env_service: Arc<dyn EnvServiceFn + Send + Sync>,
  hub_service: Arc<dyn HubService + Send + Sync>,
  data_service: Arc<dyn DataService + Send + Sync>,
  auth_service: Arc<dyn AuthService + Send + Sync>,
}

impl AppService {
  pub fn new(
    env_service: Arc<dyn EnvServiceFn + Send + Sync>,
    hub_service: HfHubService,
    data_service: LocalDataService,
    auth_service: KeycloakAuthService,
  ) -> Self {
    Self {
      env_service,
      hub_service: Arc::new(hub_service),
      data_service: Arc::new(data_service),
      auth_service: Arc::new(auth_service),
    }
  }
}

impl AppServiceFn for AppService {
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
