use crate::{
  db::DbService, AuthService, CacheService, DataService, EnvService, HubService, SecretService,
  SessionService,
};
use derive_new::new;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait AppService: std::fmt::Debug + Send + Sync {
  fn env_service(&self) -> Arc<dyn EnvService>;

  fn data_service(&self) -> Arc<dyn DataService>;

  fn hub_service(&self) -> Arc<dyn HubService>;

  fn auth_service(&self) -> Arc<dyn AuthService>;

  fn db_service(&self) -> Arc<dyn DbService>;

  fn session_service(&self) -> Arc<dyn SessionService>;

  fn secret_service(&self) -> Arc<dyn SecretService>;

  fn cache_service(&self) -> Arc<dyn CacheService>;
}

#[allow(clippy::too_many_arguments)]
#[derive(Clone, Debug, new)]
pub struct DefaultAppService {
  env_service: Arc<dyn EnvService>,
  hub_service: Arc<dyn HubService>,
  data_service: Arc<dyn DataService>,
  auth_service: Arc<dyn AuthService>,
  db_service: Arc<dyn DbService>,
  session_service: Arc<dyn SessionService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
}

impl AppService for DefaultAppService {
  fn env_service(&self) -> Arc<dyn EnvService> {
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

  fn cache_service(&self) -> Arc<dyn CacheService> {
    self.cache_service.clone()
  }
}
