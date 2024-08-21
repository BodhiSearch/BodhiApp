use super::{
  data_service::DataService, hub_service::HubService, AuthService, EnvServiceFn, SessionService,
  secret_service::SecretService,
};
use crate::db::DbService;
use derive_new::new;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait AppServiceFn: std::fmt::Debug + Send + Sync {
  fn env_service(&self) -> Arc<dyn EnvServiceFn>;

  fn data_service(&self) -> Arc<dyn DataService>;

  fn hub_service(&self) -> Arc<dyn HubService>;

  fn auth_service(&self) -> Arc<dyn AuthService>;

  fn db_service(&self) -> Arc<dyn DbService>;

  fn session_service(&self) -> Arc<dyn SessionService>;

  fn secret_service(&self) -> Arc<dyn SecretService>;
}

#[derive(Clone, Debug, new)]
pub struct AppService {
  env_service: Arc<dyn EnvServiceFn + Send + Sync>,
  hub_service: Arc<dyn HubService + Send + Sync>,
  data_service: Arc<dyn DataService + Send + Sync>,
  auth_service: Arc<dyn AuthService + Send + Sync>,
  db_service: Arc<dyn DbService + Send + Sync>,
  session_service: Arc<dyn SessionService + Send + Sync>,
  secret_service: Arc<dyn SecretService + Send + Sync>,
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

  fn db_service(&self) -> Arc<dyn DbService> {
    self.db_service.clone()
  }

  fn session_service(&self) -> Arc<dyn SessionService> {
    self.session_service.clone()
  }

  fn secret_service(&self) -> Arc<dyn SecretService> {
    self.secret_service.clone()
  }
}