use crate::{
  db::{DbService, TimeService},
  inference::LocalLlama,
  AccessRequestService, AiApiClientFactory, ApiModelService, AuthService, CacheService,
  ConcurrencyService, DataService, DownloadService, HealthRegistry, HubService, McpService,
  ModelRouterService, NetworkService, QueueProducer, SessionService, SettingService, TenantService,
  TokenService,
};
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait AppService: std::fmt::Debug + Send + Sync {
  fn setting_service(&self) -> Arc<dyn SettingService>;

  fn data_service(&self) -> Arc<dyn DataService>;

  fn hub_service(&self) -> Arc<dyn HubService>;

  fn auth_service(&self) -> Arc<dyn AuthService>;

  fn db_service(&self) -> Arc<dyn DbService>;

  fn session_service(&self) -> Arc<dyn SessionService>;

  fn tenant_service(&self) -> Arc<dyn TenantService>;

  fn cache_service(&self) -> Arc<dyn CacheService>;

  fn time_service(&self) -> Arc<dyn TimeService>;

  fn ai_api_client_factory(&self) -> Arc<dyn AiApiClientFactory>;

  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService>;

  fn queue_producer(&self) -> Arc<dyn QueueProducer>;

  fn network_service(&self) -> Arc<dyn NetworkService>;

  fn access_request_service(&self) -> Arc<dyn AccessRequestService>;

  fn mcp_service(&self) -> Arc<dyn McpService>;

  fn token_service(&self) -> Arc<dyn TokenService>;

  fn local_llama(&self) -> Option<Arc<dyn LocalLlama>>;

  fn api_model_service(&self) -> Arc<dyn ApiModelService>;

  fn model_router_service(&self) -> Arc<dyn ModelRouterService>;

  fn health_registry(&self) -> Arc<dyn HealthRegistry>;

  fn download_service(&self) -> Arc<dyn DownloadService>;

  fn queue_status(&self) -> String {
    self.queue_producer().queue_status()
  }
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
  tenant_service: Arc<dyn TenantService>,
  cache_service: Arc<dyn CacheService>,
  time_service: Arc<dyn TimeService>,
  ai_api_client_factory: Arc<dyn AiApiClientFactory>,
  concurrency_service: Arc<dyn ConcurrencyService>,
  queue_producer: Arc<dyn QueueProducer>,
  network_service: Arc<dyn NetworkService>,
  access_request_service: Arc<dyn AccessRequestService>,
  mcp_service: Arc<dyn McpService>,
  token_service: Arc<dyn TokenService>,
  local_llama: Option<Arc<dyn LocalLlama>>,
  api_model_service: Arc<dyn ApiModelService>,
  model_router_service: Arc<dyn ModelRouterService>,
  health_registry: Arc<dyn HealthRegistry>,
  download_service: Arc<dyn DownloadService>,
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

  fn tenant_service(&self) -> Arc<dyn TenantService> {
    self.tenant_service.clone()
  }

  fn cache_service(&self) -> Arc<dyn CacheService> {
    self.cache_service.clone()
  }

  fn time_service(&self) -> Arc<dyn TimeService> {
    self.time_service.clone()
  }

  fn ai_api_client_factory(&self) -> Arc<dyn AiApiClientFactory> {
    self.ai_api_client_factory.clone()
  }

  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService> {
    self.concurrency_service.clone()
  }

  fn queue_producer(&self) -> Arc<dyn QueueProducer> {
    self.queue_producer.clone()
  }

  fn network_service(&self) -> Arc<dyn NetworkService> {
    self.network_service.clone()
  }

  fn access_request_service(&self) -> Arc<dyn AccessRequestService> {
    self.access_request_service.clone()
  }

  fn mcp_service(&self) -> Arc<dyn McpService> {
    self.mcp_service.clone()
  }

  fn token_service(&self) -> Arc<dyn TokenService> {
    self.token_service.clone()
  }

  fn local_llama(&self) -> Option<Arc<dyn LocalLlama>> {
    self.local_llama.clone()
  }

  fn api_model_service(&self) -> Arc<dyn ApiModelService> {
    self.api_model_service.clone()
  }

  fn model_router_service(&self) -> Arc<dyn ModelRouterService> {
    self.model_router_service.clone()
  }

  fn health_registry(&self) -> Arc<dyn HealthRegistry> {
    self.health_registry.clone()
  }

  fn download_service(&self) -> Arc<dyn DownloadService> {
    self.download_service.clone()
  }
}
