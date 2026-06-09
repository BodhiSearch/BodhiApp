use crate::{
  auth::AuthContextError,
  db::{DbService, TimeService},
  AccessRequestService, AiApiClientFactory, AppService, AuthContext, AuthScopedAiApiClientFactory,
  AuthScopedApiModelService, AuthScopedDataService, AuthScopedDownloadService,
  AuthScopedMcpService, AuthScopedModelRouterService, AuthScopedTenantService,
  AuthScopedTokenService, AuthScopedUserAccessRequestService, AuthScopedUserService, AuthService,
  CacheService, ConcurrencyService, DataService, HealthRegistry, HubService, NetworkService,
  QueueProducer, SessionService, SettingService, TenantService,
};
use std::sync::Arc;

pub struct AuthScopedAppService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedAppService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  pub fn app_service(&self) -> &Arc<dyn AppService> {
    &self.app_service
  }

  pub fn auth_context(&self) -> &AuthContext {
    &self.auth_context
  }

  pub fn require_user_id(&self) -> Result<&str, AuthContextError> {
    self.auth_context.require_user_id()
  }

  pub fn client_id(&self) -> Option<&str> {
    self.auth_context.client_id()
  }

  pub fn require_client_id(&self) -> Result<&str, AuthContextError> {
    self.auth_context.require_client_id()
  }

  pub fn tenant_id(&self) -> Option<&str> {
    self.auth_context.tenant_id()
  }

  pub fn require_tenant_id(&self) -> Result<&str, AuthContextError> {
    self.auth_context.require_tenant_id()
  }

  /// Returns an auth-scoped token service. Each call clones the inner Arc and AuthContext,
  /// so bind to a local variable if calling multiple methods: `let svc = auth_scope.tokens();`
  pub fn tokens(&self) -> AuthScopedTokenService {
    AuthScopedTokenService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn mcps(&self) -> AuthScopedMcpService {
    AuthScopedMcpService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn users(&self) -> AuthScopedUserService {
    AuthScopedUserService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn api_models(&self) -> AuthScopedApiModelService {
    AuthScopedApiModelService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn model_routers(&self) -> AuthScopedModelRouterService {
    AuthScopedModelRouterService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn downloads(&self) -> AuthScopedDownloadService {
    AuthScopedDownloadService::new(self.app_service.clone(), self.auth_context.clone())
  }

  /// Returns an auth-scoped data service. Each call clones the inner Arc and AuthContext,
  /// so bind to a local variable if calling multiple methods: `let svc = auth_scope.data();`
  pub fn data(&self) -> AuthScopedDataService {
    AuthScopedDataService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn user_access_requests(&self) -> AuthScopedUserAccessRequestService {
    AuthScopedUserAccessRequestService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn settings(&self) -> Arc<dyn SettingService> {
    self.app_service.setting_service()
  }

  pub fn tenants(&self) -> AuthScopedTenantService {
    AuthScopedTenantService::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn auth_flow(&self) -> Arc<dyn AuthService> {
    self.app_service.auth_service()
  }

  pub fn network(&self) -> Arc<dyn NetworkService> {
    self.app_service.network_service()
  }

  pub fn sessions(&self) -> Arc<dyn SessionService> {
    self.app_service.session_service()
  }

  pub fn db(&self) -> Arc<dyn DbService> {
    self.app_service.db_service()
  }

  pub fn hub(&self) -> Arc<dyn HubService> {
    self.app_service.hub_service()
  }

  /// Auth-scoped — auto-injects tenant_id/user_id when constructing Liberty clients
  /// via `for_resolved`.
  pub fn ai_api(&self) -> AuthScopedAiApiClientFactory {
    AuthScopedAiApiClientFactory::new(self.app_service.clone(), self.auth_context.clone())
  }

  pub fn time(&self) -> Arc<dyn TimeService> {
    self.app_service.time_service()
  }

  // Legacy pass-through accessors for tests and routes not yet migrated to the
  // short-name factory methods above.
  pub fn data_service(&self) -> Arc<dyn DataService> {
    self.app_service.data_service()
  }

  pub fn hub_service(&self) -> Arc<dyn HubService> {
    self.app_service.hub_service()
  }

  pub fn setting_service(&self) -> Arc<dyn SettingService> {
    self.app_service.setting_service()
  }

  pub fn time_service(&self) -> Arc<dyn TimeService> {
    self.app_service.time_service()
  }

  pub fn db_service(&self) -> Arc<dyn DbService> {
    self.app_service.db_service()
  }

  pub fn session_service(&self) -> Arc<dyn SessionService> {
    self.app_service.session_service()
  }

  pub fn network_service(&self) -> Arc<dyn NetworkService> {
    self.app_service.network_service()
  }

  pub fn ai_api_client_factory(&self) -> Arc<dyn AiApiClientFactory> {
    self.app_service.ai_api_client_factory()
  }

  pub fn health_registry(&self) -> Arc<dyn HealthRegistry> {
    self.app_service.health_registry()
  }

  pub fn queue_producer(&self) -> Arc<dyn QueueProducer> {
    self.app_service.queue_producer()
  }

  pub fn tenant_service(&self) -> Arc<dyn TenantService> {
    self.app_service.tenant_service()
  }

  /// Non-auth-scoped passthrough. See [`AccessRequestService`] doc comment for rationale.
  /// All methods on this service manage their own tenant/user context — they are not
  /// filtered by AuthContext's tenant_id/user_id.
  pub fn access_request_service(&self) -> Arc<dyn AccessRequestService> {
    self.app_service.access_request_service()
  }

  pub fn cache_service(&self) -> Arc<dyn CacheService> {
    self.app_service.cache_service()
  }

  pub fn auth_service(&self) -> Arc<dyn AuthService> {
    self.app_service.auth_service()
  }

  pub fn concurrency_service(&self) -> Arc<dyn ConcurrencyService> {
    self.app_service.concurrency_service()
  }

  pub fn queue_status(&self) -> String {
    self.app_service.queue_status()
  }
}
