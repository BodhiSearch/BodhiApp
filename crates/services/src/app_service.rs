use crate::{
  db::{DbService, TimeService},
  AiApiService, AuthService, CacheService, ConcurrencyService, DataService, HubService,
  SecretService, SessionService, SettingService,
};
use objs::LocalizationService;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait AppService: std::fmt::Debug + Send + Sync {
  fn setting_service(&self) -> Arc<dyn SettingService>;

  fn data_service(&self) -> Arc<dyn DataService>;

  fn hub_service(&self) -> Arc<dyn HubService>;

  fn auth_service(&self) -> Arc<dyn AuthService>;

  fn db_service(&self) -> Arc<dyn DbService>;

  fn session_service(&self) -> Arc<dyn SessionService>;

  fn secret_service(&self) -> Arc<dyn SecretService>;

  fn cache_service(&self) -> Arc<dyn CacheService>;

  fn localization_service(&self) -> Arc<dyn LocalizationService>;

  fn time_service(&self) -> Arc<dyn TimeService>;

  fn ai_api_service(&self) -> Arc<dyn AiApiService>;

  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService>;
}

#[allow(clippy::too_many_arguments)]
#[derive(Clone, Debug, derive_new::new)]
pub struct DefaultAppService {
  env_service: Arc<dyn SettingService>,
  hub_service: Arc<dyn HubService>,
  data_service: Arc<dyn DataService>,
  auth_service: Arc<dyn AuthService>,
  db_service: Arc<dyn DbService>,
  session_service: Arc<dyn SessionService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  localization_service: Arc<dyn LocalizationService>,
  time_service: Arc<dyn TimeService>,
  ai_api_service: Arc<dyn AiApiService>,
  concurrency_service: Arc<dyn ConcurrencyService>,
}

impl AppService for DefaultAppService {
  fn setting_service(&self) -> Arc<dyn SettingService> {
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

  fn localization_service(&self) -> Arc<dyn LocalizationService> {
    self.localization_service.clone()
  }

  fn time_service(&self) -> Arc<dyn TimeService> {
    self.time_service.clone()
  }

  fn ai_api_service(&self) -> Arc<dyn AiApiService> {
    self.ai_api_service.clone()
  }

  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService> {
    self.concurrency_service.clone()
  }
}
