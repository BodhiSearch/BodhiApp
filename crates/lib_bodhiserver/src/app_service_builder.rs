use crate::BootstrapError;
use objs::{EnvType, IoError};
use services::{
  db::{DbCore, DbPool, DbService, DefaultTimeService, SqliteDbService, TimeService},
  hash_key, AccessRequestService, AiApiService, AppInstance, AppInstanceService, AppService,
  AuthService, BootstrapParts, CacheService, DataService, DefaultAccessRequestService,
  DefaultAiApiService, DefaultAppInstanceService, DefaultAppService, DefaultExaService,
  DefaultMcpService, DefaultNetworkService, DefaultSessionService, DefaultSettingService,
  DefaultToolService, ExaService, HfHubService, HubService, InMemoryQueue, KeycloakAuthService,
  KeyringStore, LocalConcurrencyService, LocalDataService, McpService, MokaCacheService,
  NetworkService, QueueConsumer, QueueProducer, RefreshWorker, SessionService, SettingService,
  SystemKeyringStore, ToolService, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, HF_TOKEN, PROD_DB,
};
use std::path::PathBuf;
use std::sync::Arc;

const SECRET_KEY: &str = "secret_key";

pub struct AppServiceBuilder {
  bootstrap_parts: Option<BootstrapParts>,
  // Externally injectable services (for testing).
  // To add injection for a new service, follow the cache_service pattern.
  time_service: Option<Arc<dyn TimeService>>,
  cache_service: Option<Arc<dyn CacheService>>,
}

impl std::fmt::Debug for AppServiceBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppServiceBuilder")
      .field(
        "bootstrap_parts",
        &self.bootstrap_parts.as_ref().map(|_| "<BootstrapParts>"),
      )
      .field(
        "time_service",
        &self.time_service.as_ref().map(|_| "<TimeService>"),
      )
      .field(
        "cache_service",
        &self.cache_service.as_ref().map(|_| "<CacheService>"),
      )
      .finish()
  }
}

impl AppServiceBuilder {
  pub fn new(bootstrap_parts: BootstrapParts) -> Self {
    Self {
      bootstrap_parts: Some(bootstrap_parts),
      time_service: None,
      cache_service: None,
    }
  }

  /// Sets the time service.
  pub fn time_service(mut self, service: Arc<dyn TimeService>) -> Result<Self, BootstrapError> {
    if self.time_service.is_some() {
      return Err(BootstrapError::ServiceAlreadySet(
        "time_service".to_string(),
      ));
    }
    self.time_service = Some(service);
    Ok(self)
  }

  /// Sets the cache service.
  pub fn cache_service(mut self, service: Arc<dyn CacheService>) -> Result<Self, BootstrapError> {
    if self.cache_service.is_some() {
      return Err(BootstrapError::ServiceAlreadySet(
        "cache_service".to_string(),
      ));
    }
    self.cache_service = Some(service);
    Ok(self)
  }

  pub async fn build(mut self) -> Result<DefaultAppService, BootstrapError> {
    let time_service = self.get_or_build_time_service();

    let parts = self
      .bootstrap_parts
      .take()
      .ok_or(BootstrapError::MissingBootstrapParts)?;

    let is_production = parts.system_settings.iter().any(|s| {
      s.key == BODHI_ENV_TYPE && s.value.as_str() == Some(&EnvType::Production.to_string())
    });
    let encryption_key_value = parts.env_wrapper.var(BODHI_ENCRYPTION_KEY).ok();
    let encryption_key = build_encryption_key(is_production, encryption_key_value).await?;

    let db_service = Self::build_db_service(
      parts.bodhi_home.join(PROD_DB),
      time_service.clone(),
      encryption_key.clone(),
    )
    .await?;

    let setting_service: Arc<dyn SettingService> =
      Arc::new(DefaultSettingService::from_parts(parts, db_service.clone()));

    let hub_service = Self::build_hub_service(&setting_service).await?;
    let app_instance_service: Arc<dyn AppInstanceService> =
      Arc::new(DefaultAppInstanceService::new(db_service.clone()));
    let data_service = Self::build_data_service(hub_service.clone(), db_service.clone());
    let session_service = Self::build_session_service(&setting_service).await?;
    let cache_service = self.get_or_build_cache_service();
    let auth_service = Self::build_auth_service(&setting_service).await;
    let ai_api_service = Self::build_ai_api_service(db_service.clone());
    let concurrency_service = Self::build_concurrency_service();
    let tool_service = Self::build_tool_service(db_service.clone(), time_service.clone());
    let access_request_service = Self::build_access_request_service(
      &setting_service,
      db_service.clone(),
      auth_service.clone(),
      time_service.clone(),
    )
    .await;
    let network_service = Self::build_network_service();
    let mcp_service = Self::build_mcp_service(db_service.clone(), time_service.clone());

    // Create queue and spawn refresh worker
    let queue = Arc::new(InMemoryQueue::new());
    let is_processing = queue.get_is_processing();
    let queue_producer: Arc<dyn QueueProducer> = queue.clone();
    let queue_consumer: Arc<dyn QueueConsumer> = queue;

    // Spawn refresh worker in background
    let worker = RefreshWorker::new(
      queue_consumer,
      hub_service.clone(),
      data_service.clone(),
      db_service.clone(),
      is_processing,
    );
    tokio::spawn(async move {
      worker.run().await;
    });

    // Build and return the complete app service
    let app_service = DefaultAppService::new(
      setting_service,
      hub_service,
      data_service,
      auth_service,
      db_service,
      session_service,
      app_instance_service,
      cache_service,
      time_service,
      ai_api_service,
      concurrency_service,
      queue_producer,
      tool_service,
      network_service,
      access_request_service,
      mcp_service,
    );
    Ok(app_service)
  }

  /// Builds the hub service.
  async fn build_hub_service(
    setting_service: &Arc<dyn SettingService>,
  ) -> Result<Arc<dyn HubService>, BootstrapError> {
    let hf_cache = setting_service.hf_cache().await;
    let hf_token = setting_service.get_env(HF_TOKEN).await;
    let hub_service = HfHubService::new_from_hf_cache(hf_cache, hf_token, true)
      .map_err(|err| BootstrapError::Io(IoError::from(err)))?;
    Ok(Arc::new(hub_service))
  }

  /// Builds the data service.
  fn build_data_service(
    hub_service: Arc<dyn HubService>,
    db_service: Arc<dyn DbService>,
  ) -> Arc<dyn DataService> {
    Arc::new(LocalDataService::new(hub_service, db_service))
  }

  /// Gets or builds the time service.
  fn get_or_build_time_service(&mut self) -> Arc<dyn TimeService> {
    if let Some(service) = self.time_service.take() {
      return service;
    }

    Arc::new(DefaultTimeService)
  }

  /// Builds the database service using a direct db_path.
  async fn build_db_service(
    db_path: PathBuf,
    time_service: Arc<dyn TimeService>,
    encryption_key: Vec<u8>,
  ) -> Result<Arc<dyn DbService>, BootstrapError> {
    let app_db_pool = DbPool::connect(&format!("sqlite:{}", db_path.display())).await?;
    let db_service = SqliteDbService::new(app_db_pool, time_service, encryption_key);
    db_service.migrate().await?;
    Ok(Arc::new(db_service))
  }

  /// Builds the session service from the session DB URL.
  /// Detects backend from URL scheme (sqlite:// or postgres://).
  async fn build_session_service(
    setting_service: &Arc<dyn SettingService>,
  ) -> Result<Arc<dyn SessionService>, BootstrapError> {
    let url = setting_service.session_db_url().await;
    let session_service = DefaultSessionService::connect(&url).await?;
    Ok(Arc::new(session_service))
  }

  /// Gets or builds the cache service.
  fn get_or_build_cache_service(&mut self) -> Arc<dyn CacheService> {
    if let Some(service) = self.cache_service.take() {
      return service;
    }

    Arc::new(MokaCacheService::default())
  }

  /// Builds the auth service.
  async fn build_auth_service(setting_service: &Arc<dyn SettingService>) -> Arc<dyn AuthService> {
    let auth_url = setting_service.auth_url().await;
    let auth_realm = setting_service.auth_realm().await;
    Arc::new(KeycloakAuthService::new(
      &setting_service.version().await,
      auth_url,
      auth_realm,
    ))
  }

  /// Builds the AI API service.
  fn build_ai_api_service(db_service: Arc<dyn DbService>) -> Arc<dyn AiApiService> {
    Arc::new(DefaultAiApiService::with_db_service(db_service))
  }

  /// Builds the concurrency service.
  fn build_concurrency_service() -> Arc<dyn services::ConcurrencyService> {
    Arc::new(LocalConcurrencyService::new())
  }

  /// Builds the tool service.
  fn build_tool_service(
    db_service: Arc<dyn DbService>,
    time_service: Arc<dyn TimeService>,
  ) -> Arc<dyn ToolService> {
    let exa_service: Arc<dyn ExaService> = Arc::new(DefaultExaService::new());
    Arc::new(DefaultToolService::new(
      db_service,
      exa_service,
      time_service,
    ))
  }

  /// Builds the access request service.
  async fn build_access_request_service(
    setting_service: &Arc<dyn SettingService>,
    db_service: Arc<dyn DbService>,
    auth_service: Arc<dyn AuthService>,
    time_service: Arc<dyn TimeService>,
  ) -> Arc<dyn AccessRequestService> {
    let frontend_url = setting_service.public_server_url().await;
    Arc::new(DefaultAccessRequestService::new(
      db_service,
      auth_service,
      time_service,
      frontend_url,
    ))
  }

  /// Builds the network service.
  fn build_network_service() -> Arc<dyn NetworkService> {
    Arc::new(DefaultNetworkService)
  }

  /// Builds the MCP service.
  fn build_mcp_service(
    db_service: Arc<dyn DbService>,
    time_service: Arc<dyn TimeService>,
  ) -> Arc<dyn McpService> {
    let mcp_client = Arc::new(mcp_client::DefaultMcpClient::new());
    Arc::new(DefaultMcpService::new(db_service, mcp_client, time_service))
  }
}

/// Builds the encryption key given production flag and optional env-provided key value.
async fn build_encryption_key(
  is_production: bool,
  encryption_key_value: Option<String>,
) -> Result<Vec<u8>, BootstrapError> {
  let app_suffix = if is_production { "" } else { " - Dev" };
  let app_name = format!("Bodhi App{app_suffix}");
  if let Some(ref key) = encryption_key_value {
    if key == "your-strong-encryption-key-here" {
      return Err(BootstrapError::PlaceholderValue(key.to_string()));
    }
  }
  let encryption_key = encryption_key_value
    .map(|key| Ok(hash_key(&key)))
    .unwrap_or_else(|| {
      SystemKeyringStore::new(&app_name)
        .get_or_generate(SECRET_KEY)
        .map_err(BootstrapError::from)
    })?;
  Ok(encryption_key)
}

pub async fn build_app_service(
  bootstrap_parts: BootstrapParts,
) -> Result<DefaultAppService, BootstrapError> {
  AppServiceBuilder::new(bootstrap_parts).build().await
}

pub async fn update_with_option(
  service: &Arc<dyn AppService>,
  instance: Option<&AppInstance>,
) -> Result<(), BootstrapError> {
  if let Some(instance) = instance {
    service
      .app_instance_service()
      .create_instance(
        &instance.client_id,
        &instance.client_secret,
        instance.status.clone(),
      )
      .await?;
  }
  Ok(())
}

#[cfg(test)]
#[path = "test_app_service_builder.rs"]
mod test_app_service_builder;
